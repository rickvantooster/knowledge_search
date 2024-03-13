use std::{ path::{Path, PathBuf}, sync::{Arc, OnceLock, RwLock}};

pub mod base;
pub mod json_model;
pub mod invertedmodel;
use base::Model;


use self::{json_model::JsonModel, invertedmodel::InvertedModel};

pub static GLOB_CORPUS: OnceLock<Arc<RwLock<CorpusModel>>> = OnceLock::new();

unsafe impl Sync for CorpusModel {}
unsafe impl Send for CorpusModel {}


enum ModelKind {
    JsonModel(JsonModel),
    InvertedModel(InvertedModel)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModelType {
    Json,
    Inverted

}

pub fn path_to_index_name(path: &Path, kind: ModelType) -> PathBuf {
    let possible_name = path.file_name();
    if possible_name.is_none() {
        eprintln!("Invalid directory name {}", path.display());
        std::process::exit(3);
    }
    let name = path.file_name().unwrap().clone();
    let mut index_name = name.to_str().unwrap().to_string();

    match kind {
        ModelType::Json => index_name.push_str(".index.json"),
        ModelType::Inverted => index_name.push_str(".index.bin")
    };

    PathBuf::from(index_name)

}

#[derive(Debug)]
pub struct CorpusModel{
    inner: Box<dyn Model>,
    kind: ModelType
}


impl CorpusModel {
    pub fn new_json_model(index_path: &Path) -> Self {

        let model = JsonModel::from_disk(&index_path).unwrap_or(JsonModel::new());
        CorpusModel { inner: Box::new(model), kind: ModelType::Json}
    }

    pub fn new_inverted_model(index_path: &Path) -> Self {
        let model = InvertedModel::from_disk(&index_path).unwrap_or(InvertedModel::new());
        CorpusModel { inner: Box::new(model), kind: ModelType::Inverted }

    }

}

impl Model for CorpusModel {
    fn add_document(&mut self, path: PathBuf, content: &[char]) {
        self.inner.add_document(path, content);
    }

    fn remove_document(&mut self, path: PathBuf) {
        self.inner.remove_document(path);
    }

    fn needs_reindex(&self, path: &PathBuf) -> Result<bool, base::ReindexError> {
        self.inner.needs_reindex(path)
    }

    fn store_with_name(&mut self, index_path: &PathBuf) {
        self.inner.store_with_name(index_path);
    }

    fn store(&mut self) {
        self.inner.store();
    }

    fn from_disk(index_path: &Path) -> Option<Self> where Self: Sized {
        //self.inner.from_disk(index_path)
        unreachable!()
    }

    fn search_simple(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        self.inner.search_simple(query)
    }

    fn search_singular_exact(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        if self.kind == ModelType::Inverted {
            self.inner.search_simple(query)
        }else{
            self.inner.search_singular_exact(query)

        }
    }

    fn search_phrase(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        if self.kind == ModelType::Inverted {
            self.inner.search_simple(query)
        }else{
            self.inner.search_phrase(query)

        }
    }

    fn contains_tokens_sequential(&self, pos: usize, qt: Vec<String>, doc: &base::Document) -> bool {
        unreachable!();
    }

    fn assert_next_token_pos(&self, doc: &base::Document, pos: usize, t: &String) -> bool {
        unreachable!();
        todo!()
    }

    fn docs_with_all_terms(&self, qt: &[String]) -> Option<Vec<PathBuf>> {
        unreachable!();
        todo!()
    }

    fn get_documents(&self) -> base::Documents {
        unreachable!();
    }

    fn delete_removed_files(&mut self) {
        self.inner.delete_removed_files();
    }

    fn add_document_batched(&mut self, batch: Vec<(PathBuf, Vec<char>)>, dir: PathBuf) {
        self.inner.add_document_batched(batch, dir);
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}

