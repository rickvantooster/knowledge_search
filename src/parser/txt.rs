use std::{path::Path, io::{BufReader, Read}};

use super::error::ParserError;

pub fn parse_txt(file: &Path) -> Result<Vec<char>, ParserError> {
    let mut result: Vec<u8> = Vec::new();
    let file = std::fs::File::open(file)?;
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut result)?;
    let result = String::from_utf8_lossy(&result);

    

    //Ok(result.replace('\r', "").replace('\n', " "))
    Ok(result.replace('\r', "").replace('\n', " ").chars().collect())
}
