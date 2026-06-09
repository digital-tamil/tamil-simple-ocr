use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub fn correct_tamil_text_with_ollama(
    current_tamil_txt: &str,
    previous_text: Option<&str>,
    model: &str,
    url: &str,
) -> Result<String> {
    let client = reqwest::blocking::Client::new();

    let prompt = match previous_text {
        Some(tamil_prev_txt) => format!("<|turn>system
<|think|>
You are an expert Tamil editor, classical literature scholar, and precise OCR correction assistant.
Your task is to analyze raw OCR text from a CURRENT page and correct it, ensuring seamless syntactic and narrative flow from a PREVIOUS corrected page.

Inputs you will receive:
1. Corrected Tamil text from the previous page (used only for context and to handle words or sentences split across page boundaries).
2. Raw, uncorrected OCR text from the current page.

Linguistic Correction Guidelines:
- Correct spelling, spacing, character errors, and grammatical inconsistencies on the CURRENT page.
- Do not alter the core semantic meaning, tone, or style of the original Tamil text.
- Pay close attention to the transition between pages. If a word or sentence was cut off or split at the page boundary, ensure it is resolved correctly and grammatically in the corrected current page output.

Response Constraints:
- Output ONLY the corrected Tamil text for the CURRENT page.
- Do NOT output or repeat the previous page's text.
- Do NOT write any conversational preambles, introductory statements, explanations, or notes (in English or Tamil).<turn|>
<|turn>user
Please correct the current page's OCR text based on the provided guidelines:

<previous_page_text>
{tamil_prev_txt}
</previous_page_text>

<current_page_ocr>
{current_tamil_txt}
</current_page_ocr><turn|>
<|turn>model"),

        None => format!("<|turn>system
<|think|>
You are an expert Tamil language philologist, classical literature scholar, and a precise OCR correction assistant.
Your task is to analyze and correct Tamil text extracted from a PDF via Tesseract OCR.

The input may contain typical OCR errors, such as:
- Misread characters, glyph splits, and character-matching errors.
- Incorrect spacing or missing word boundaries.
- Disconnected words or sentences split across line or page boundaries.

Linguistic Correction Guidelines:
1. Reconstruct and repair words or sentences that are broken across line or page boundaries to maintain continuity.
2. Correct spelling, grammar, and character errors without altering the core semantic meaning.
3. Preserve the original old/classical Tamil vocabulary and register; do not modernize classical or literary words, but do correct spelling or morphological errors.
4. Leverage deep literary context to resolve ambiguous or heavily distorted OCR characters.

Response Constraints:
- Output ONLY the corrected Tamil text.
- Do NOT provide any conversational preambles, introductory sentences, markdown headers, explanations, or notes (in English or Tamil).<turn|>
<|turn>user
Please analyze and correct the following OCR-extracted Tamil text based on the system guidelines:

<paragraph_input>
{current_tamil_txt}
</paragraph_input><turn|>
<|turn>model"),
    };

    let payload = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
    };

    let response = client
        .post(format!("{}/api/generate", url))
        .json(&payload)
        .send()
        .context("Failed to connect to the Ollama server")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Ollama server returned an error: {}",
            response.status()
        ));
    }

    let res_body: OllamaResponse = response
        .json()
        .context("Failed to parse the response JSON from Ollama")?;
    println!("{:?}", res_body.response.trim().to_string());
    Ok(res_body.response.trim().to_string())
}
