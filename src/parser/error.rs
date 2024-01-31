use std::fmt::Display;

use pdf_extract::OutputError;
use zip::result::ZipError;

pub enum ParserError {
    ZipError(ZipError),
    IOError(std::io::Error),
    XmlError(xml::reader::Error),
    PdfError(OutputError)
}

impl From<std::io::Error> for ParserError {
    fn from(value: std::io::Error) -> Self {
        ParserError::IOError(value)
    }
}

impl From<ZipError> for ParserError {
    fn from(value: ZipError) -> Self {
        ParserError::ZipError(value)
    }
}

impl From<xml::reader::Error> for ParserError{
    fn from(value: xml::reader::Error) -> Self {
        ParserError::XmlError(value)
    }

}
impl From<OutputError> for ParserError {
    fn from(value: OutputError) -> Self {
        ParserError::PdfError(value)
    }

}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::ZipError(e) => writeln!(f, "ZipError: {e}"),
            ParserError::IOError(e) => writeln!(f, "IOError: {e}"),
            ParserError::XmlError(e) => writeln!(f, "XmlError: {e}"),
            ParserError::PdfError(e) => writeln!(f, "PdfError: {e}"),
        }
    }
}
