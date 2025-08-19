use std::{error::Error, process::Command, fs, path::Path};

/// Extracts all text from the PDF at `path` using pdftotext, falling back to Tesseract OCR if needed.
pub fn extract_text_from_pdf(path: &str) -> Result<String, Box<dyn Error>> {
    println!("Starting PDF text extraction from: {}", path);
    let pdf_path = Path::new(path);
    let txt_path = pdf_path.with_extension("txt");

    // 1. Try pdftotext (Poppler)
    let pdftotext_status = Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg(&txt_path)
        .status();

    let mut text = String::new();
    if let Ok(status) = pdftotext_status {
        if status.success() && txt_path.exists() {
            text = fs::read_to_string(&txt_path).unwrap_or_default();
            // Clean up txt file
            let _ = fs::remove_file(&txt_path);
        }
    }

    // Check for error patterns or empty output
    let error_pattern = text.contains("?Identity-H") || text.contains("Unimplemented") || text.trim().is_empty();
    if error_pattern {
        println!("pdftotext failed or returned bad output. Falling back to Tesseract OCR...");
        text = extract_text_with_tesseract(path)?;
    }

    // Normalize whitespace
    let normalized = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if normalized.trim().is_empty() {
        return Err("No text could be extracted from the PDF. The PDF might be image-based, corrupted, or use unsupported encoding.".into());
    }

    println!("Text extraction completed. Normalized length: {} characters", normalized.len());
    println!("First 200 characters of normalized text: '{}'", &normalized[..normalized.len().min(200)]);
    Ok(normalized)
}

/// Fallback: Use Tesseract OCR to extract text from each page of the PDF.
fn extract_text_with_tesseract(pdf_path: &str) -> Result<String, Box<dyn Error>> {
    use tempfile::tempdir;

    let dir = tempdir()?;
    let image_prefix = dir.path().join("page");
    let image_prefix_str = image_prefix.to_str().unwrap();

    // 1. Convert PDF to images (one per page) using 'pdftoppm'
    let status = Command::new("pdftoppm")
        .arg("-png")
        .arg(pdf_path)
        .arg(image_prefix_str)
        .status()?;
    if !status.success() {
        return Err("Failed to convert PDF to images for OCR".into());
    }

    // 2. Run tesseract on each image
    let mut full_text = String::new();
    let mut page_num = 1;
    loop {
        let image_path = dir.path().join(format!("page-{}.png", page_num));
        if !image_path.exists() {
            break;
        }
        let txt_path = dir.path().join(format!("page-{}.txt", page_num));
        let status = Command::new("tesseract")
            .arg(&image_path)
            .arg(&txt_path.with_extension("").to_str().unwrap())
            .status()?;
        if status.success() && txt_path.exists() {
            let page_text = fs::read_to_string(&txt_path).unwrap_or_default();
            full_text.push_str(&page_text);
            full_text.push(' ');
            let _ = fs::remove_file(&txt_path);
        }
        let _ = fs::remove_file(&image_path);
        page_num += 1;
    }
    dir.close()?;
    Ok(full_text)
}
