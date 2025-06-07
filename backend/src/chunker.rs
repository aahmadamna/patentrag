/// Splits `text` into overlapping chunks.
/// Each chunk has `chunk_size` words and overlaps `overlap` words with the previous.
///
/// # Example
/// let chunks = chunk_text("a b c ...", 800, 200);
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    // Split on whitespace to get all words
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();

    // Sliding window
    let mut start = 0;
    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk = words[start..end].join(" ");
        chunks.push(chunk);
        if end == words.len() {
            break;
        }
        // Advance by chunk_size - overlap
        start += chunk_size.saturating_sub(overlap);
    }

    chunks
}
