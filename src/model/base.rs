use std::{collections::HashMap, fmt::{Debug, Display}, path::{Path, PathBuf}, time::{SystemTime, SystemTimeError}};

use serde::{Deserialize, Serialize};

pub trait Model: Debug {

    fn add_document(&mut self, path: PathBuf, content: &[char]);

    #[allow(dead_code)]
    fn remove_document(&mut self, path: PathBuf);

    fn needs_reindex(&self, path: &PathBuf) -> Result<bool, ReindexError>;
    fn store_with_name(&mut self, index_path: &PathBuf);

    fn store(&mut self);

    fn from_disk(index_path: &Path) -> Option<Self> where Self: Sized;

    fn search_simple(&self, query: &[char]) -> Vec<(PathBuf, f64)>;

    fn search_singular_exact(&self, query: &[char]) -> Vec<(PathBuf, f64)>;
    
    fn search_phrase(&self, query: &[char]) -> Vec<(PathBuf, f64)>;

    fn contains_tokens_sequential(&self, pos: usize, qt: Vec<String>, doc: &Document) -> bool;

    fn assert_next_token_pos(&self, doc: &Document, pos: usize, t: &String) -> bool;

    fn docs_with_all_terms(&self, qt: &[String]) -> Option<Vec<PathBuf>>;

    fn get_documents(&self) -> Documents;

    fn delete_removed_files(&mut self);

    fn add_document_batched(&mut self, batch: Vec<(PathBuf, Vec<char>)>, dir: PathBuf);

    fn reset(&mut self);
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TermInner {
    pub count: usize,
    pub positions: Vec<usize>
}

impl TermInner {
    pub fn new(position: usize) -> Self{
        TermInner { count: 1, positions: vec![position] }
    }
}


pub type TermFrequency = HashMap<String, TermInner>;
pub type Documents = HashMap<PathBuf, Document>;

pub type DocumentFrequency = HashMap<String, usize>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Document {
    pub tf: TermFrequency,
    pub tf_stemmed: TermFrequency,
    pub count: usize,
    pub last_updated: usize

}

#[inline(always)]
pub fn calculate_tf(d: &Document, t: &String, stemmed: bool) -> f64 {
    let n = d.count as f64;
    let tf = if stemmed {
        &d.tf_stemmed
    }else{
        &d.tf

    };

    let m = if let Some(ti) = tf.get(t) {
        ti.count as f64
    }else{
        0.0_f64
    };

    m / n


}
#[inline(always)]
pub fn calculate_idf(t: &String, n: usize, df: &DocumentFrequency) -> f64 {
    let n = n as f64;
    let f = df.get(t).cloned().unwrap_or(1) as f64;
    (n / f).log10()


}

#[inline(always)]
pub fn get_last_modified(path: &PathBuf) -> Result<SystemTime, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(modified)

}

#[derive(Debug)]
pub enum ReindexError {
    SystemTimeError(SystemTimeError),
    IOError(std::io::Error)
}

impl From<std::io::Error> for ReindexError {

    fn from(value: std::io::Error) -> Self {
        ReindexError::IOError(value)
    }
}

impl From<SystemTimeError> for ReindexError {

    fn from(value: SystemTimeError) -> Self {
        ReindexError::SystemTimeError(value)
    }
}

impl Display for ReindexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReindexError::SystemTimeError(x) => write!(f, "SystemTimeError: {x}"),
            ReindexError::IOError(x) => write!(f, "IOError: {x}"),
        }
        
    }
}

