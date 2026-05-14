use anyhow::{Context, Result};
use clap::Parser;
use pdfium_render::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use tesseract::Tesseract;

/// Simple tool to extract Tamil text from PDFs via OCR.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the PDF file to perform OCR on
    #[arg(short, long, default_value = "test.pdf",  value_hint=clap::ValueHint::FilePath)]
    pdf_path: String,

    /// Language code for Tesseract (e.g., 'tam' for Tamil)
    #[arg(short, long, default_value = "tam")]
    lang: String,

    /// Output text file path
    #[arg(short, long, default_value = "tamil_pdf_extracted_text_parallel.txt", value_hint=clap::ValueHint::FilePath)]
    output: String,

    /// Enable verbose/debug logging
    #[arg(short, long,action= clap::ArgAction::SetTrue)]
    debug: bool,
}

// ... [Args struct and main function stay the same] ...

fn ocr_pdf(pdf_path: &str, lang: &str, args_debug: bool) -> Result<String> {
    // 1. Initialize Pdfium
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

    // Evaluate once outside the closure
    let tessdata_path = format!("{}/src/tessdata", env!("CARGO_MANIFEST_DIR"));
    let lower_lang = lang.to_lowercase();

    // MUTEXES: Protect the FFI boundaries from parallel race conditions!
    let render_mutex = Mutex::new(());
    let tess_init_mutex = Mutex::new(());

    // 3. Iterate over each page in the PDF in parallel
    let parallel_results: Result<Vec<String>> = pages_vec.par_iter().map(|page| {

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
        // _guard is dropped here, other threads can now render safely.

        let tesseract = {
            let _guard = tess_init_mutex.lock().unwrap(); // Lock libc locale mutation
            Tesseract::new(Some(tessdata_path.as_str()), Some(lower_lang.as_str()))
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

    // 4. Proper error handling
    let txts = parallel_results?;
    let full_text = txts.join("\n");

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

    let extracted_text = ocr_pdf(&args.pdf_path, &args.lang, args.debug)?;

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
