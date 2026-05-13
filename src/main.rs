use anyhow::{Context, Result};
use clap::Parser;
use pdfium_render::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tesseract::Tesseract;

/// Simple tool to extract Tamil text from PDFs via OCR.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the PDF file to perform OCR on
    #[arg(short, long)]
    pdf_path: String,

    /// Language code for Tesseract (e.g., 'tam' for Tamil)
    #[arg(short, long, default_value = "tam")]
    lang: String,

    /// Output text file path
    #[arg(short, long, default_value = "tamil_pdf_extracted_text.txt")]
    output: String,
}

fn ocr_pdf(pdf_path: &str, lang: &str) -> Result<String> {
    // 1. Initialize Pdfium. Late binding automatically checks the execution folder, then system PATH.
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

    let mut full_text = String::new();
    let parrallel_results: Vec<String> = document
        .pages()
        .iter()
        .enumerate()
        .par_bridge()
        .map(|(pagenum, page)| {
            todo!("Need to add Parallelism ");
            pagenum.to_string()
        })
        .collect();
    // 3. Iterate over each page in the PDF
    for (page_num, page) in document.pages().iter().enumerate() {
        // Zoom factor of 2 (roughly 144 DPI on a standard 72 DPI page).
        // For ancient/degraded Tamil text, you could push this to 3.0 if accuracy is low.
        const ZOOM: f32 = 2.0;
        let target_width = (page.width().value * ZOOM) as i32;

        let render_config = PdfRenderConfig::new().set_target_width(target_width);

        // 4. Isolate page processing to neatly catch page-specific panics/errors.
        let result = (|| -> Result<String> {
            // Render the page to memory and convert to RGB8 image
            let bitmap = page.render_with_config(&render_config)?;
            let image = bitmap.as_image()?.into_rgb8();

            let width = image.width() as i32;
            let height = image.height() as i32;
            const BYTES_PER_PIXEL: i32 = 3;
            let bytes_per_line = width * BYTES_PER_PIXEL;
            let frame_data = image.as_raw();
            let tessdata_path = format!("{}/src/tessdata", env!("CARGO_MANIFEST_DIR"));

            // 5. Initialize Tesseract, load frame memory directly, and get text
            let mut tesseract = Tesseract::new(Some(&tessdata_path), Some(&lang.to_lowercase()))
                .context("Failed to initialize Tesseract. Make sure Tesseract and 'tam' lang data are installed.")?
                .set_frame(frame_data, width, height, BYTES_PER_PIXEL, bytes_per_line)
                .context("Failed to set frame for Tesseract")?;

            let text = tesseract
                .get_text()
                .context("Failed to extract text from image")?;
            Ok(text)
        })();

        // 6. Output and append results
        match result {
            Ok(text) => {
                println!("--- Page {} ---", page_num + 1);
                println!("{}", text);
                full_text.push_str(&text);
                full_text.push('\n');
            }
            Err(e) => {
                eprintln!("Error processing page {}: {:#}", page_num + 1, e);
            }
        }
    }

    Ok(full_text)
}

fn main() -> Result<()> {
    // We use clap to allow overriding defaults easily without changing the code
    let args = Args::parse();

    println!("Starting OCR process...");

    if !Path::new(&args.pdf_path).exists() {
        eprintln!("Error: The file '{}' was not found.", args.pdf_path);
        std::process::exit(1);
    }

    let extracted_text = ocr_pdf(&args.pdf_path, &args.lang)?;

    let mut file = File::create(&args.output)
        .with_context(|| format!("Failed to create output file: {}", args.output))?;

    file.write_all(extracted_text.as_bytes())
        .with_context(|| format!("Failed to write to output file: {}", args.output))?;

    println!(
        "\nOCR process complete. Extracted text saved to '{}'",
        args.output
    );

    Ok(())
}
