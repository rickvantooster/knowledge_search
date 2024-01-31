use std::{path::Path, fs::File};

use xml::EventReader;
use zip::{ZipArchive, result::ZipError, read::ZipFile};

use super::error::ParserError;



pub fn get_docx_text_manual(path: &Path) -> Result<Vec<char>, ParserError> {
    let mut result = String::new();
    let mut zip = ZipArchive::new(File::open(path)?)?;

    let document = zip_read("word/document.xml".to_string(), &mut zip)?;
    let footnotes = zip_option_read("word/footnotes.xml".to_string(), &mut zip)?;
    let endnotes = zip_option_read("word/endnotes.xml".to_string(), &mut zip)?;
    let comments = zip_option_read("word/comments.xml".to_string(), &mut zip)?;

    let headers = zip_option_read_multiple("word/header".to_string(), &mut zip)?;
    let footers = zip_option_read_multiple("word/footer".to_string(), &mut zip)?;

    result.push_str(document.as_str());
    if let Some(txt) = footnotes {
        result.push_str(txt.as_str());

    }

    if let Some(txt) = endnotes {
        result.push_str(txt.as_str());

    }
    if let Some(txt) = comments {
        result.push_str(txt.as_str());

    }

    for (_, txt) in headers {
        result.push_str(txt.as_str());

    }
    for (_, txt) in footers {
        result.push_str(txt.as_str());

    }


    /*
    println!("document: {document}");
    println!("footnotes: {:?}", footnotes);
    println!("endnotes: {:?}", endnotes);
    println!("comments: {:?}", comments);
    println!("headers: {:?}", headers);
    println!("footer: {:?}", footer);
    */


    Ok(result.chars().collect())
}

fn zip_parse_text(parser: EventReader<ZipFile>) -> Result<String, ParserError> {
    let mut result = String::new();

    let mut in_text = false;
    let mut in_choice = false;


    for elem in parser {
        match elem {
            Ok(xml::reader::XmlEvent::StartElement { name, ..}) =>{
                if name.to_string().ends_with("}mc:Choice") {
                    in_choice = true;
                }
                if name.to_string().ends_with("}w:t") {
                    in_text = true;

                }
            },
            Ok(xml::reader::XmlEvent::EndElement { name}) => {
                if name.to_string().ends_with("}mc:Choice") {
                    in_choice = false;
                }
                if name.to_string().ends_with("}w:t") {
                    in_text = false;

                }
            },
            Ok(xml::reader::XmlEvent::Characters(chars)) => {
                if in_text && !in_choice {
                //if in_text {
                    result.push_str(chars.as_str());
                    result.push(' ');
                }

            }
            Ok(_) => (),
            //eprintln!("element of type {elem:?} is being ignored"),
            Err(e) =>{
                return Err(ParserError::XmlError(e))
            },
        }

    }
    Ok(result)
}

fn zip_read(file: String, zip: &mut ZipArchive<File>) -> Result<String, ParserError> {

    let file = zip.by_name(file.as_str())?;
    let parser = EventReader::new(file);
    zip_parse_text(parser)
}

fn zip_option_read(file: String, zip: &mut ZipArchive<File>) -> Result<Option<String>, ParserError>{
    match zip.by_name(file.as_str()) {
        Err(ZipError::FileNotFound) => Ok(None),
        Err(_) => Ok(None),
        Ok(file) => {
            let parser = EventReader::new(file);

            //file.read_to_string(&mut buffer).ok()?;
            let temp = zip_parse_text(parser)?;
            Ok(Some(temp))
        }
    }

}

fn zip_option_read_multiple(file: String, zip: &mut ZipArchive<File>) -> Result<Vec<(String, String)>, ParserError> {
        let names: Vec<String> = zip.file_names().map(|x| x.to_string()).collect();
        let name_and_value: Result<Vec<(String, String)>, ParserError> = names
            .iter()
            .filter(|n| n.contains(file.as_str()))
            .filter_map(|f| {
                zip.by_name(f).ok().map(|file| {
                
                    let parser = EventReader::new(file);
                    let temp = zip_parse_text(parser)?;

                    Ok((f.to_string(), temp))
                })
            })
            .collect();
        name_and_value

}
