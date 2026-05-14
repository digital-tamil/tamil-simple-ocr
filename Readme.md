

# 📝 Tamil Simple OCR (Rust-Powered)

![Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust&style=for-the-badge)
![Performance](https://img.shields.io/badge/Performance-Ultra--Fast-brightgreen?style=for-the-badge)
![AI](https://img.shields.io/badge/Focus-Tamil%20LLM-blue?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Mac%20%7C%20Linux%20%7C%20Windows-lightgrey?style=for-the-badge)

A high-performance, parallel-processed OCR engine specifically designed for digitizing Tamil literature and **Siddhar manuscripts** to accelerate LLM fine-tuning.

> **"தேமதுரத் தமிழோசை உலகமெலாம் பரவும் வகை செய்தல் வேண்டும்"**
> <br/>
> *(We must ensure the honey-sweet sound of Tamil spreads across the entire world.)*
> <br/>
> — **Mahakavi Bharathiyar** —


---

## 🌟 Why This Exists?
Most existing Tamil OCR tools are built on Python wrappers for Tesseract, which struggle with large-scale processing. For fine-tuning LLMs on vast libraries of Tamil literature, speed is non-negotiable. 

This tool was built to:
- **Accelerate Data Extraction:** Converting thousands of pages of Tamil PDF/Books into clean text datasets.
- **Siddhar Literature Focus:** Optimized for the unique formatting and classical Tamil text found in Siddhar books.
- **Performance First:** Leverages Rust’s memory safety and parallel processing (multi-core) to outshine standard Tesseract implementations.

---

## ⚡ Technical Highlights
*   **Parallel Execution:** Uses Rust's thread-safety to process multiple PDF pages simultaneously.
*   **LLVM Optimized:** The Mac binary is pre-compiled with full LLVM optimizations for peak silicon performance.
*   **Memory Efficient:** Zero-cost abstractions ensure the tool stays lightweight even with 500+ page PDFs.

---

## 🚀 Quick Start

### 🍏 For Mac Users (Recommended)
We provide a pre-built, high-speed binary for macOS.
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/tamil-simple-ocr.git
   cd tamil-simple-ocr/build
   ```
2. Run the binary directly:
   ```bash
   ./tamil-simple-ocr --help
   ```

### 💻 Other OS (Linux / Windows)
You can run it directly using the Rust toolchain:
```bash
cargo run --release -- --help
```

---

## 🛠 Prerequisites & Installation

### 1. Install PDFium (Required)
This tool requires the `pdfium` library to handle PDF rendering.

*   **macOS:** 
    ```bash
    brew install pdfium
    ```
*   **Ubuntu/Linux:** 
    ```bash
    sudo apt-get install libpdfium-dev
    ```
*   **Windows:** 
    Download the `pdfium.dll` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) and place it in your project root or `System32`.

### 2. Rust Toolchain (Non-Mac users)
If you are compiling from source, ensure you have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🎯 Future Roadmap
We are evolving this from a "Simple OCR" to an "AI-First Digitization Suite":

- [ ] **Neural Network Integration:** Moving beyond traditional OCR to custom-trained Tamil character recognition models.
- [ ] **Gemma LLM Correction:** Integrating **Google's Gemma** models to analyze context and automatically correct OCR errors on the fly.
- [ ] **Siddhar Context Engine:** A specific module to understand the archaic Tamil terminology used in ancient medical/spiritual texts.

---

## 🤝 Connect with the Developer
Building the future of Tamil AI. Let's collaborate!

| Platform | Profile |
| :--- | :--- |
| **Personal Developer** | [![Instagram](https://img.shields.io/badge/Instagram-E4405F?logo=instagram&logoColor=white)](https://www.instagram.com/sanjaiyan_dev/) |
| **Tamil AI Community** | [![Instagram](https://img.shields.io/badge/Instagram-E4405F?logo=instagram&logoColor=white)](https://www.instagram.com/tamil.ai.llm/) |

---

### 📜 License
*Designed with ❤️ for the தமிழ் Language.*

---

