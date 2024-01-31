use std::{path::Path, io::{BufReader, Read}};

use super::error::ParserError;

pub fn parse_txt(file: &Path) -> Result<Vec<char>, ParserError> {
    let mut result = String::new();
    let file = std::fs::File::open(file)?;
    let mut reader = BufReader::new(file);
    reader.read_to_string(&mut result)?;

    

    //Ok(result.replace('\r', "").replace('\n', " "))
    Ok(result.replace('\r', "").replace('\n', " ").chars().collect())
}
