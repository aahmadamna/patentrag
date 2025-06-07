use lopdf::Document;
use std::{error::Error, fs::File, io::BufReader};

/// Extracts all text from the PDF at `path`, in page order,
/// collapsing runs of whitespace into single spaces.
pub fn extract_text_from_pdf(path: &str) -> Result<String, Box<dyn Error>> {
    // Load the PDF document
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let doc = Document::load_from(&mut reader)?;

    // Collect text from each page
    let mut full_text = String::new();
    for (_page_num, page_id) in doc.get_pages() {
        // Extract text for this page
        let page_text = doc.extract_text(&[page_id])?;
        // Append with a space separator
        full_text.push_str(&page_text);
        full_text.push(' ');
    }

    // Normalize whitespace: collapse any runs of whitespace into a single space
    let normalized = full_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Ok(normalized)
}
