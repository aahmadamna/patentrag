mod ingest; mod chunker;

use std::env;
use ingest::extract_text_from_pdf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple CLI: expect `ingest <pdf_path> <patent_id>`
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();

    if command == "ingest" {
        let pdf_path = args.next().expect("Missing PDF path");
        let patent_id = args.next().expect("Missing patent ID");

        println!("Ingesting PDF '{}' as patent ID '{}'", pdf_path, patent_id);

        // 1. Extract all text
        let full_text = extract_text_from_pdf(&pdf_path)?;
        println!("✅ Extracted {} characters of text", full_text.len());
        //` 2. Chunk the text
        let chunks = chunker::chunk_text(&full_text, 800, 200);
        println!("✅ Created {} chunks", chunks.len());

        // (Next steps: chunk & insert into DB…)
    } else {
        eprintln!("Usage: patentrag ingest <pdf_path> <patent_id>");
    }

    Ok(())
}
