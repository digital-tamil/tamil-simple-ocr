use anyhow::{Context, Result};
use clap::Parser;
use pdfium_render::prelude::*;
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
    #[arg(short, long,action= clap::ArgAction::SetTrue)]
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
    // Initialize Pdfium
    let pdfium_bindings =
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .context(
                "Failed to bind to Pdfium library. Please ensure the pdfium binary is available.",
            )?;

    let pdfium = Pdfium::new(pdfium_bindings);

    // 2. Open the PDF document
    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .with_context(|| format!("Failed to open PDF file: {}", pdf_path))?;

    let pages_vec: Vec<_> = document.pages().iter().collect();
    let total_pages = pages_vec.len();

    // Evaluate once outside the closure
    let tessdata_path = if std::env::var("CARGO").is_ok() {
        // If running via 'cargo run', look in your project's src folder
        format!("{}/src/tessdata", env!("CARGO_MANIFEST_DIR"))
    } else {
        // If running the standalone binary, look for the folder next to the EXE
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push("tessdata");
        path.to_string_lossy().into_owned()
    };
    let lang = lang.to_lowercase();

    // MUTEXES: Protect the FFI boundaries from parallel race conditions!
    let render_mutex = Mutex::new(());
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

    const BATCH_SIZE: usize = 10;

    for chunk_start in (0..total_pages).step_by(BATCH_SIZE) {
        let chunk_end = std::cmp::min(chunk_start + BATCH_SIZE, total_pages);
        println!(
            "Processing batch: pages {} to {} of {}...",
            chunk_start + 1,
            chunk_end,
            total_pages
        );

        let batch_pages = &pages_vec[chunk_start..chunk_end];
        let batch_ocr_results: Result<Vec<String>> = batch_pages.par_iter().map(|page| {

        let (frame_data, width, height, bytes_per_line) = {
            let _guard = render_mutex.lock().unwrap(); // Lock FFI
            const ZOOM: f32 = 2.0;
            let target_width = (page.width().value * ZOOM) as i32;

            let render_config = PdfRenderConfig::new().set_target_width(target_width);
            let bitmap = page.render_with_config(&render_config).context("Failed to render page")?;
            let image = bitmap.as_image().context("Failed to unwrap image")?.into_rgb8();

            let width = image.width() as i32;
            let height = image.height() as i32;
            const BYTES_PER_PIXEL: i32 = 3;
            let bytes_per_line = width * BYTES_PER_PIXEL;
            (image.into_raw(), width, height, bytes_per_line)
        };

        let tesseract = {
            let _guard = tess_init_mutex.lock().unwrap(); // Lock libc locale mutation
            Tesseract::new(Some(tessdata_path.as_str()), Some(lang.as_str()))
                .context("Failed to initialize Tesseract. Make sure Tesseract and lang data are installed.")?
        };

        // Setting the frame and extracting text is 100% thread-safe per instance.
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
    }).collect();

        let batch_txts = batch_ocr_results?;

        let final_batch_texts = if let Some(model) = ollama_model {
            // Part B: Concurrently run Ollama (exactly 2 requests at a time)
            if args_debug {
                println!("Running Ollama correction on batch with a concurrency of 2...");
            }

            // Construct previous page context vectors using raw OCR text to avoid sequential blockages
            let mut raw_contexts: Vec<Option<String>> = Vec::with_capacity(batch_txts.len());
            for i in 0..batch_txts.len() {
                if i == 0 {
                    raw_contexts.push(previous_raw_text.clone());
                } else {
                    raw_contexts.push(Some(batch_txts[i - 1].clone()));
                }
            }

            // Execute the Ollama requests inside the capped thread pool
            let corrected_results: Result<Vec<String>> = ollama_pool.install(|| {
                batch_txts
                    .par_iter()
                    .zip(raw_contexts.par_iter())
                    .map(|(current_text, prev_context)| {
                        if current_text.trim().is_empty() {
                            return Ok(current_text.clone());
                        }

                        // Call the module function we created earlier
                        ollama::correct_tamil_text_with_ollama(
                            current_text,
                            prev_context.as_deref(),
                            model,
                            ollama_url,
                        )
                    })
                    .collect()
            });

            match corrected_results {
                Ok(corrected_list) => {
                    // Update our sliding raw text marker to the end of this batch
                    previous_raw_text = batch_txts.last().cloned();
                    corrected_list
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Batch correction failed: {}. Falling back to raw OCR text.",
                        e
                    );
                    previous_raw_text = batch_txts.last().cloned();
                    batch_txts
                }
            }
        } else {
            batch_txts
        };

        // Part C: Write corrected results to disk immediately & flush heap allocations
        let batch_output = final_batch_texts.join("\n");
        file.write_all(batch_output.as_bytes())
            .context("Failed to write batch data to file")?;
        file.write_all(b"\n")?;
        file.flush().context("Failed to flush data to disk")?;

        // At this point in the loop, temporary raw image buffers and large intermediate strings
        // go out of scope and are cleaned up from the heap.
    }

    Ok(())
}

fn main() -> Result<()> {
    // We use clap to allow overriding defaults easily without changing the code
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
