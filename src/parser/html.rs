use std::{path::Path, io::BufReader, char};

use xml::EventReader;

use super::error::ParserError;

pub fn get_html_text(path: &Path) -> Result<Vec<char>, ParserError> {

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    //reader.read_to_string(&mut buffer).unwrap();



    Ok(html2text::from_read(reader, usize::MAX).replace('\r', "").replace('\n', " ").chars().collect())
}

pub fn get_xhtml(path: &Path) -> Result<Vec<char>, ParserError> {
    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(file);

    let event_reader = EventReader::new(reader);
    let mut content = String::new();

    for evn in event_reader {
        match evn {
            Ok(xml::reader::XmlEvent::Characters(chars)) => {
                content.push_str(chars.as_str());
                content.push(' ');
            },
            Err(e) => {
                return Err(ParserError::XmlError(e))

            },
            _ => ()
        }

    }

    Ok(content.replace('\r', "").replace('\n', " ").chars().collect())
}
