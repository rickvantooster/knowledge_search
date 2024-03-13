use core::f64;
use std::{collections::HashMap, fs::File, io::{BufReader, BufWriter, Write}, path::PathBuf, time::UNIX_EPOCH, usize};

use serde::{Deserialize, Serialize};

use crate::lexer::Lexer;

use super::base::{Model, get_last_modified};


pub type TF = f64;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvertedModelDocumentMeta {
    path: PathBuf,
    //terms: HashSet<String>, //easy lookup for file deletion. Tho it can probably be removed as it
                            //would be better to just iterate over each term and check if it
                            //includes the document we are trying to delete
    last_updated: usize

}
//TODO: split up model interface in to 2 seperate ones
// One for model management (adding, deleting and updating files)
// And one for search queries.
// That way we might be able to reduce the memory footprint as 
// we don't need the documents_meta hashmap in memory when we want to do search queries

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvertedModel {
    path: Option<PathBuf>,
    count: usize,
    term_frequency: HashMap<String, HashMap<PathBuf, TF>>,
    documents_meta: HashMap<PathBuf, InvertedModelDocumentMeta>,

}

impl InvertedModel {
    pub fn new() -> InvertedModel{
        InvertedModel {
            term_frequency: HashMap::new(),
            documents_meta: HashMap::new(),
            path: None,
            count: 0
        }

    }

}

impl Model for InvertedModel {
    fn add_document(&mut self, path: PathBuf, content: &[char]) {
        if self.documents_meta.contains_key(&path) {
            self.count -= 1;
            for (_, freq) in &mut self.term_frequency {
                if freq.contains_key(&path) {
                    freq.remove(&path).unwrap();
                }
            }
        }

        let doc_meta = InvertedModelDocumentMeta{
            path: path.clone(),
            //terms: HashSet::new(),
            last_updated: get_last_modified(&path).unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize,
        };

        
        let lexer = Lexer::new_stemmed(content);
        let terms: Vec<String> = lexer.collect();
        //doc_meta.terms = HashSet::from_iter(terms.iter().cloned());
        let count: usize = terms.len();

        let mut tf = HashMap::<String, usize>::new();

        for term in terms {
            if let Some(entry) = tf.get_mut(&term) {
                *entry += 1;
            }else{
                tf.insert(term.clone(), 1);
            }
        }

        

        for (term, freq) in tf {
            if let Some(entry) = self.term_frequency.get_mut(&term) {
                if !entry.contains_key(&path) {
                    entry.insert(path.clone(), freq as f64 / count as f64);
                }
            }else{
                let mut inner_map: HashMap<PathBuf, TF> = HashMap::new();
                inner_map.insert(path.clone(), freq as f64 / count as f64);
                self.term_frequency.insert(term.clone(), inner_map);
            }


        }

        self.documents_meta.insert(path.clone(), doc_meta);
        self.count += 1;



    }

    fn remove_document(&mut self, path: PathBuf) {
        if let Some(curr) = self.documents_meta.remove(&path) {
            for (_, freq) in &mut self.term_frequency {
                if freq.contains_key(&path) {
                    freq.remove(&path).unwrap();
                }
            }
        }
    }

    fn needs_reindex(&self, path: &PathBuf) -> Result<bool, super::base::ReindexError> {
        if !self.documents_meta.contains_key(path) {
            return Ok(true)
        }

        let last_updated = self.documents_meta.get(path).unwrap().last_updated;

        let file_last_updated = get_last_modified(path)?.duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;

        Ok(file_last_updated > last_updated)


    }

    // We currently store the index file as a json for debugging purposes but later op we 
    // will replace it with a bincoded file with varint enabled for a smaller index file size.
    fn store_with_name(&mut self, index_path: &PathBuf) {
        let index_file = File::create(index_path).unwrap();
        self.path = Some(index_path.clone());
        let bytes = bincode::serialize(&self).unwrap();
        let mut writer = BufWriter::new(index_file);
        writer.write_all(&bytes).unwrap();
        
        //serde_json::to_writer(BufWriter::new(index_file), &self).unwrap();
    }

    fn store(&mut self) {
        match self.path.clone() {
            Some(p) => self.store_with_name(&p),
            None => ()

        };

    }

    fn from_disk(index_path: &std::path::Path) -> Option<Self> where Self: Sized {
        let index_file = File::open(index_path);
        if index_file.is_err() {
            return None;

        }

        let buf_reader = BufReader::new(index_file.unwrap());
        let decoded: Option<InvertedModel> = bincode::deserialize_from(buf_reader).unwrap();
        decoded
    }

    fn search_simple(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        let search_query: Vec<String> = Lexer::new_stemmed(query).collect();
        let mut results: HashMap<PathBuf, f64> = HashMap::new();

        for term in search_query {
            if let Some(entry) = self.term_frequency.get(&term) {
                for (file, tf) in entry {
                    let tfidf = tf * (self.documents_meta.len() as f64 / entry.len() as f64).log10();

                    if let Some(weight) = results.get_mut(file) {
                        *weight += tfidf;
                    }else{
                        results.insert(file.clone(), tfidf);
                    }
                }
            }
        }

        let mut weighted: Vec<(PathBuf, f64)> = results.iter().map(|(path, weight)| (path.clone(), weight.clone())).filter(|(_, w)| w > &0.0 ).collect();

        weighted.sort_by(|(_, rank1), (_, rank2)| {
            rank2.partial_cmp(rank1).unwrap()

        });

        weighted


    }

    fn search_singular_exact(&self, _query: &[char]) -> Vec<(PathBuf, f64)> {
        unreachable!()
    }

    fn search_phrase(&self, _query: &[char]) -> Vec<(PathBuf, f64)> {
        unreachable!()
    }

    fn contains_tokens_sequential(&self, _pos: usize, _qt: Vec<String>, _doc: &super::base::Document) -> bool {
        unreachable!()
    }

    fn assert_next_token_pos(&self, _doc: &super::base::Document, _pos: usize, _t: &String) -> bool {
        unreachable!()
    }

    fn docs_with_all_terms(&self, _qt: &[String]) -> Option<Vec<PathBuf>> {
        unreachable!()
    }

    fn get_documents(&self) -> super::base::Documents {
        todo!()
    }

    fn delete_removed_files(&mut self) {
        let to_remove: Vec<PathBuf> = self.documents_meta.iter().filter_map(|(k, _)|{
            if !k.exists() {
                Some(k.to_path_buf())
            }else{
                None
            }
        }).collect();

        for(_, list) in &mut self.term_frequency {
            to_remove.iter().for_each(|file|{
                list.remove(file);
            });


        }


        //unreachable!();
    }

    fn add_document_batched(&mut self, batch: Vec<(PathBuf, Vec<char>)>, _dir: PathBuf) {
        for (file, content) in batch {
            self.add_document(file, &content);

        }
    }

    fn reset(&mut self) {
        todo!()
    }
}
