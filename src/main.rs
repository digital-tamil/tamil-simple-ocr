use anyhow::{Context, Result};
use clap::Parser;
use pdf_oxide::PdfDocument;
use pdf_oxide::rendering::{RenderOptions, render_page};
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use tesseract::Tesseract;

mod ollama;

/// Simple tool to extract Tamil text from PDFs via OCR.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the PDF file to perform OCR on
    #[arg(short, long, value_hint=clap::ValueHint::FilePath)]
    pdf_path: String,

    /// Language code for Tesseract (e.g., 'tam' for Tamil)
    #[arg(short, long, default_value = "tam")]
    lang: String,

    /// Output text file path
    #[arg(short, long, default_value = "tamil_pdf_extracted_text.txt", value_hint=clap::ValueHint::FilePath)]
    output: String,

    /// Enable verbose/debug logging
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[arg(long, default_value = "http://localhost:11434", value_hint=clap::ValueHint::Url)]
    ollama_url: String,

    #[arg(long, default_value = "gemma4")]
    ollama_model: Option<String>,
}

fn ocr_pdf(
    pdf_path: &str,
    lang: &str,
    args_debug: bool,
    ollama_model: Option<&str>,
    ollama_url: &str,
    output_path: &str,
) -> Result<()> {
    //Open the Pdf
    let doc = PdfDocument::open(pdf_path)
        .with_context(|| format!("Failed to open PDF file: {}", pdf_path))?;

    let total_pages = doc.page_count()?;

    // Evaluate once outside the closure
    let tessdata_path = if std::env::var("CARGO").is_ok() {
        format!("{}/src/tessdata", env!("CARGO_MANIFEST_DIR"))
    } else {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push("tessdata");
        path.to_string_lossy().into_owned()
    };
    let lang = lang.to_lowercase();

    // MUTEXES: Protect the FFI boundaries from parallel race conditions!
    let tess_init_mutex = Mutex::new(());

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .with_context(|| format!("Failed to open output file: {}", output_path))?;

    // Create a dedicated Rayon thread pool capped at 2 threads for Ollama calls
    let ollama_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .context("Failed to build concurrent Ollama thread pool")?;

    let mut previous_raw_text: Option<String> = None;

    // Instantiate a single shared, thread-safe HTTP Client once.
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(None)
        .build()
        .context("Failed to initialize HTTP client")?;

    const BATCH_SIZE: usize = 10;

    for chunk_start in (0..total_pages).step_by(BATCH_SIZE) {
        let chunk_end = std::cmp::min(chunk_start + BATCH_SIZE, total_pages);
        println!(
            "Processing batch: pages {} to {} of {}...",
            chunk_start + 1,
            chunk_end,
            total_pages
        );

        let page_indices: Vec<usize> = (chunk_start..chunk_end).collect();
        let batch_ocr_results: Result<Vec<String>> = page_indices
            .par_iter()
            .map(|&page_idx| {
                // Configure rendering options. DPI 300
                let opts = RenderOptions::with_dpi(300);

                // render_page parses and draws the PDF page in pure Rust (using tiny-skia).
                let rendered_image = render_page(&doc, page_idx, &opts)
                    .context("Failed to render page using pdf_oxide")?;

                // Decode the rendered image bytes (PNG format by default) into raw RGB pixels.
                let decoded_image = image::load_from_memory(&rendered_image.data)
                    .context("Failed to decode rendered page bytes")?
                    .into_rgb8();

                let width = decoded_image.width() as i32;
                let height = decoded_image.height() as i32;
                const BYTES_PER_PIXEL: i32 = 3;
                let bytes_per_line = width * BYTES_PER_PIXEL;
                let frame_data = decoded_image.into_raw();

                let tesseract = {
                    let _guard = tess_init_mutex.lock().unwrap(); // Lock libc locale mutation
                    Tesseract::new(Some(tessdata_path.as_str()), Some(lang.as_str()))
                        .context("Failed to initialize Tesseract.")?
                };

                let mut tesseract = tesseract
                    .set_frame(&frame_data, width, height, 3, bytes_per_line)
                    .context("Failed to set frame for Tesseract")?;

                let text = tesseract
                    .get_text()
                    .context("Failed to extract text from image")?;

                if args_debug {
                    println!("Currently converted following content: {}", text);
                }

                Ok(text)
            })
            .collect();

        let batch_txts = batch_ocr_results?;

        let final_batch_texts = if let Some(model) = ollama_model {
            if args_debug {
                println!("Running Ollama correction on batch with a concurrency of 2...");
            }

            let mut raw_contexts: Vec<Option<String>> = Vec::with_capacity(batch_txts.len());
            for i in 0..batch_txts.len() {
                if i == 0 {
                    raw_contexts.push(previous_raw_text.clone());
                } else {
                    raw_contexts.push(Some(batch_txts[i - 1].clone()));
                }
            }

            // We collect straight into Vec<String> instead of Result<Vec<String>>
            let corrected_list: Vec<String> = ollama_pool.install(|| {
                batch_txts
                    .par_iter()
                    .zip(raw_contexts.par_iter())
                    .enumerate()
                    .map(|(offset, (current_text, prev_context))| {
                        let absolute_page_idx = chunk_start + offset + 1;
                        if current_text.trim().is_empty() {
                            return current_text.clone();
                        }

                        // Local error handling ensures page failures are isolated
                        match ollama::correct_tamil_text_with_ollama(
                            &client, // Pass reference to the shared pool client
                            current_text,
                            prev_context.as_deref(),
                            model,
                            ollama_url,
                        ) {
                            Ok(corrected) => corrected,
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to correct page {} with Ollama: {}. Falling back to raw text.",
                                    absolute_page_idx, e
                                );
                                current_text.clone() // Local fallback
                            }
                        }
                    })
                    .collect()
            });

            previous_raw_text = batch_txts.last().cloned();
            corrected_list
        } else {
            batch_txts
        };

        // Write corrected results to disk immediately & flush heap allocations
        let batch_output = final_batch_texts.join("\n");
        file.write_all(batch_output.as_bytes())
            .context("Failed to write batch data to file")?;
        file.write_all(b"\n")?;
        file.flush().context("Failed to flush data to disk")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Starting OCR process...");

    if !Path::new(&args.pdf_path).exists() {
        eprintln!("Error: The file '{}' was not found.", args.pdf_path);
        std::process::exit(1);
    }

    ocr_pdf(
        &args.pdf_path,
        &args.lang,
        args.debug,
        args.ollama_model.as_deref(),
        &args.ollama_url,
        &args.output,
    )?;

    println!(
        "\nOCR process complete. Extracted text saved to '{}'",
        args.output
    );

    Ok(())
}
