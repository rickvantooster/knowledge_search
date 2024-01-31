use std::path::Path;

use super::error::ParserError;

pub fn get_pdf(file: &Path) -> Result<Vec<char>, ParserError> {
    let bytes = std::fs::read(file)?;
    let out = pdf_extract::extract_text_from_mem(&bytes)?;

    Ok(out.replace('\r', "").replace('\n', " ").chars().collect())
}
