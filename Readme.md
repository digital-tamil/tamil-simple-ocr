# 📝 Tamil Simple OCR: AI-Powered Digitization Suite

![Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust&style=for-the-badge)
![Ollama](https://img.shields.io/badge/AI_Engine-Ollama-white?logo=ollama&style=for-the-badge)
![Gemma](https://img.shields.io/badge/Model-Gemma_4-blue?logo=google&style=for-the-badge)
![Tauri](https://img.shields.io/badge/Upcoming-Tauri_v2-FFC131?logo=tauri&style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Mac%20%7C%20Linux%20%7C%20Windows-lightgrey?style=for-the-badge)

A high-performance, parallel-processed OCR and AI error-correction engine specifically designed for digitizing Tamil literature and **Siddhar manuscripts**. Built to accelerate Large Language Model (LLM) fine-tuning pipelines.

> **"தேமதுரத் தமிழோசை உலகமெலாம் பரவும் வகை செய்தல் வேண்டும்"**
> <br/>
> _(We must ensure the honey-sweet sound of Tamil spreads across the entire world.)_
> <br/>
> — **Mahakavi Bharathiyar** —

---

## 🌟 Why This Exists?

Most existing Tamil OCR tools are built on Python wrappers for Tesseract, which struggle with large-scale processing. Furthermore, traditional OCR output for complex classical Tamil often results in "noisy" text.

For fine-tuning LLMs on vast libraries of Tamil literature, **speed and accuracy are non-negotiable**.

This suite was built to:

- **Accelerate Data Extraction:** Convert thousands of pages of Tamil PDFs/Books into clean text datasets using Rust's multi-core parallelism.
- **Context-Aware Error Correction:** Automatically repair misidentified classical Tamil characters using local Edge-AI (Ollama + Gemma).
- **Siddhar Literature Focus:** Optimized for the unique formatting and archaic vocabulary found in ancient Siddhar medical and spiritual texts.

---

## 🏗️ System Architecture & Pipeline

The project is structured into a 3-Phase pipeline to ensure maximum performance and accuracy:

### ✅ Phase 1: Production Baseline (Completed)

- **High-Performance Processing Engine:** Written in Rust with strict LLVM & Compiler optimizations (`opt-level=3`, `lto=true`).
- **Parallel Ingestion:** Utilizes a **Rayon Work-Stealing Pool** to process multiple image buffers simultaneously.
- **Core Extraction:** Binarization and deskewing followed by LSTM character recognition via **Tesseract Rust Bindings**, generating a raw (noisy) Tamil text stream.

### 🚧 Phase 2: Edge-AI Context Correction (Active Development)

- **Local Inference:** The raw text buffer is sent via local HTTP POST to an **Ollama Local Instance**.
- **Gemma 4 E4B Model (4.5B Effective Params):** A localized LLM analyzes the noisy Tamil text and performs **Context-Aware Token Repair**, outputting highly validated, clean Tamil text.

### 📅 Phase 3: Native Desktop Delivery (Planned)

- **Tauri Core Backend:** Wrapping the Rust processing engine into a lightweight cross-platform desktop app using **Tauri v2**.
- **Frontend UI:** A responsive HTML5/TS interface communicating with the backend via IPC commands for a seamless user experience.

![Architecture Flow](/public/Tamil_OCR_timeline.png)

---

## ⚡ Technical Highlights

- **Parallel Execution:** Uses Rust's thread-safety to process multiple PDF pages simultaneously.
- **Zero-Cost Abstractions:** Memory efficient, ensuring the tool stays lightweight even with 500+ page PDFs.
- **Private AI Execution:** The Ollama integration ensures that your dataset and document processing remain 100% local and offline.

---

## 🛠 Prerequisites & Installation

### 1. Install PDFium & Tesseract (Required)

This tool requires the `pdfium` library for PDF rendering and `tesseract` for the baseline OCR.

- **macOS:** `brew install pdfium tesseract tesseract-lang`
- **Ubuntu/Linux:** `sudo apt-get install libpdfium-dev tesseract-ocr tesseract-ocr-tam`
- **Windows:** Download the `pdfium.dll` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) and install [Tesseract for Windows](https://github.com/UB-Mannheim/tesseract/wiki).

### 2. Install Ollama (For Phase 2 AI Correction)

To enable the context-aware error correction:

1. Install [Ollama](https://ollama.com/).
2. Pull the required Gemma model:
   ```bash
   ollama run gemma
   ```
   _(Note: Ensure your local Ollama API is running on the default port `11434` for the Rust backend to communicate with it)._

### 3. Rust Toolchain (Non-Mac users)

If compiling from source:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🚀 Quick Start

### 🍏 For Mac Users (Recommended)

We provide a pre-built, high-speed binary optimized for macOS silicon.

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/tamil-simple-ocr.git
   cd tamil-simple-ocr/build
   ```
2. Start your Ollama instance in the background.
3. Run the binary directly:
   ```bash
   ./tamil-simple-ocr --help
   ```

### 💻 Other OS (Linux / Windows)

You can run the processing engine directly using the Rust toolchain:

```bash
cargo run --release -- --help
```

---

## 🎯 Future Roadmap

- [x] **Phase 1:** Multi-threaded Rust OCR pipeline (Rayon + Tesseract).
- [x] **Phase 2 (Initial):** Local LLM Integration (Ollama + Gemma) for context-aware error correction.
- [] **Phase 2 (Upcoming):** Integrate a custom fine-tuned **Gemma 4 QAT (Quantization-Aware Training) Model** specifically trained on classical Siddhar vocabulary for even higher accuracy.
- [] **Phase 3:** Build a user-friendly, accessible Desktop GUI using **Tauri JS (v2)**.

---

## 🤝 Connect with the Developer

Building the future of Tamil AI. Let's collaborate!

| Platform               | Profile                                                                                                                                |
| :--------------------- | :------------------------------------------------------------------------------------------------------------------------------------- |
| **Sanjaiyan.P**        | [![Instagram](https://img.shields.io/badge/Instagram-E4405F?logo=instagram&logoColor=white)](https://www.instagram.com/sanjaiyan_dev/) |
| **Tamil AI Community** | [![Instagram](https://img.shields.io/badge/Instagram-E4405F?logo=instagram&logoColor=white)](https://www.instagram.com/tamil.ai.llm/)  |

---

### 📜 License

_Designed with ❤️ for the தமிழ் Language._
