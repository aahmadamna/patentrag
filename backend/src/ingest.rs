use lopdf::Document;
use std::{error::Error, fs::File, io::BufReader};

/// Extracts all text from the PDF at `path`, in page order,
/// collapsing runs of whitespace into single spaces.
pub fn extract_text_from_pdf(path: &str) -> Result<String, Box<dyn Error>> {
    println!("📖 Starting PDF text extraction from: {}", path);
    
    // Load the PDF document
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let doc = Document::load_from(&mut reader)?;
    
    println!("📄 PDF loaded successfully. Pages: {}", doc.get_pages().len());

    // Collect text from each page
    let mut full_text = String::new();
    let mut page_count = 0;
    
    for (page_num, _page_id) in doc.get_pages() {
        page_count += 1;
        println!("📃 Processing page {}", page_count);
        
        // Extract text for this page
        match doc.extract_text(&[page_num]) {
            Ok(page_text) => {
                println!("📝 Page {} text length: {} characters", page_count, page_text.len());
                if page_text.len() < 50 {
                    println!("⚠️  Warning: Page {} has very little text: '{}'", page_count, page_text);
                }
                // Append with a space separator
                full_text.push_str(&page_text);
                full_text.push(' ');
            }
            Err(e) => {
                println!("❌ Error extracting text from page {}: {:?}", page_count, e);
                // Continue with other pages instead of failing completely
                full_text.push_str(&format!("[Error extracting page {}: {:?}] ", page_count, e));
            }
        }
    }

    println!("📊 Total extracted text length: {} characters", full_text.len());
    
    // Check if we got meaningful content
    if full_text.trim().is_empty() {
        println!("❌ No text extracted from PDF!");
        return Err("No text could be extracted from the PDF. The PDF might be image-based, corrupted, or use unsupported encoding.".into());
    }
    
    // Check for common error patterns
    if full_text.contains("?Identity-H") || full_text.contains("Unimplemented") {
        println!("⚠️  Warning: PDF extraction may have failed - detected error patterns in text");
        println!("📄 First 200 characters: '{}'", &full_text[..full_text.len().min(200)]);
    }

    // Normalize whitespace: collapse any runs of whitespace into a single space
    let normalized = full_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    println!("✅ Text extraction completed. Normalized length: {} characters", normalized.len());
    println!("📄 First 200 characters of normalized text: '{}'", &normalized[..normalized.len().min(200)]);

    Ok(normalized)
}
