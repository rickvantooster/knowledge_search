use std::{collections::HashMap, path::{PathBuf, Path}, fs::File, io::{BufWriter, BufReader}, time::{UNIX_EPOCH, SystemTime, SystemTimeError}, sync::{OnceLock, Arc, Mutex, RwLock}, fmt::Display};
use rayon::prelude::*;
use rust_stemmers::{Algorithm, Stemmer};
use serde::{Deserialize, Serialize};

use crate::lexer::Lexer;

use super::base::{Documents, DocumentFrequency, TermFrequency, TermInner, Model, Document, ReindexError, get_last_modified, calculate_tf, calculate_idf};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonModel {
    documents: Documents,
    df: DocumentFrequency,
    df_stemmed: DocumentFrequency,
    #[serde(skip)]
    path: Option<PathBuf>


}

impl JsonModel {
    pub fn new() -> Self {
        JsonModel { documents: Documents::new(), df: DocumentFrequency::new(), df_stemmed: DocumentFrequency::new(), path: None }
    }

    pub fn new_with_args(documents: Documents, df: DocumentFrequency, df_stemmed: DocumentFrequency, path: Option<PathBuf>) -> Self {
        JsonModel { documents, df, df_stemmed, path }
    }


}

impl Model for JsonModel {

    fn add_document(&mut self, path: PathBuf, content: &[char]) {

        let metadata = std::fs::metadata(path.clone()).unwrap();

        let lexer = Lexer::new(content);
        
        let mut term_frequency = TermFrequency::new();
        let mut tf_stemmed = TermFrequency::new();
        let stemmer = Stemmer::create(Algorithm::English);
        let mut count: usize = 0;

        for (pos, token) in lexer.enumerate() {
            let stemmed = stemmer.stem(token.as_str()).to_string();

            if let Some(t) = term_frequency.get_mut(token.as_str()){
                t.count += 1;
                t.positions.push(pos);

            }else{
                term_frequency.insert(token.clone(), TermInner::new(pos));
            }

            if let Some(t) = tf_stemmed.get_mut(&stemmed){
                t.count += 1;
                t.positions.push(pos);

            }else{
                tf_stemmed.insert(stemmed.clone(), TermInner::new(pos));
            }
            count += 1;

        }


        for t in term_frequency.keys() {
            if let Some(f) = self.df.get_mut(t){
                *f += 1;
            }else{
                self.df.insert(t.clone(), 1);
            }
        }

        for t in tf_stemmed.keys() {
            if let Some(f) = self.df_stemmed.get_mut(t){
                *f += 1;
            }else{
                self.df_stemmed.insert(t.clone(), 1);
            }
        }

        self.documents.insert(path, Document { tf: term_frequency, tf_stemmed, count, last_updated: metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize });

    }

    #[allow(dead_code)]
    fn remove_document(&mut self, path: PathBuf) {
        if let Some(d) = self.documents.remove(&path){
            for t in d.tf.keys() {
                if let Some(f) = self.df.get_mut(t) {
                    *f -= 1;

                }
            }
            for t in d.tf_stemmed.keys() {
                if let Some(f) = self.df_stemmed.get_mut(t) {
                    *f -= 1;

                }
            }

        }
    }


    fn needs_reindex(&self, path: &PathBuf) -> Result<bool, ReindexError> {
        if let Some(d) = self.documents.get(path) {
            let last_modified = get_last_modified(path)?;
            
            let meta_modified = last_modified.duration_since(UNIX_EPOCH)?.as_secs() as usize;
            Ok(meta_modified > d.last_updated)


        }else{
            Ok(true)

        }

    }

    fn store_with_name(&mut self, index_path: &PathBuf){

        let index_file = File::create(index_path).unwrap();
        self.path = Some(index_path.clone());
        
        serde_json::to_writer(BufWriter::new(index_file), &self).unwrap();
    }

    fn store(&mut self) {
        if let Some(index_path) = self.path.clone() {
            self.store_with_name(&index_path);

        }

    }

    fn from_disk(index_path: &Path) -> Option<Self> {
        let index_file = File::open(index_path);
        if index_file.is_err() {
            return None;

        }

        let buf_reader = BufReader::new(index_file.unwrap());
        let res: Result<JsonModel, serde_json::Error> = serde_json::from_reader(buf_reader);
        if let Ok(mut r) = res {
            r.path = Some(index_path.to_path_buf());
            tracing::debug!("Loaded index file {} from disk", index_path.display());
            Some(r)

        }else{
            None

        }
    }

    fn search_simple(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        let qt: Vec<String> = Lexer::new_stemmed(query).collect();

        let mut result: Vec<(PathBuf, f64)> = self.documents.iter().filter_map(|(p, d)|{
            let mut rank = 0_f64;
            qt.iter().for_each(|t|{

                let tf = calculate_tf(d, t, true);
                let idf = calculate_idf(t, self.documents.len(), &self.df);
                rank += tf * idf;

            });
            if rank > 0.0 {
                Some((p.clone(), rank))
            }else{
                None
            }

        }).collect();


        result.sort_by(|(_, rank1), (_, rank2)| {
            rank2.partial_cmp(rank1).unwrap()

        });
        //result.reverse();

        result
    }

    fn search_singular_exact(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        let qt: Vec<String> = Lexer::new(query).collect();

        let docs_with_terms: Vec<PathBuf> = match self.docs_with_all_terms(&qt){
            Some(docs) => docs,
            None => return Vec::<(PathBuf, f64)>::new()

        };

        let mut result: Vec<(PathBuf, f64)> = docs_with_terms.iter().filter_map(|p|{
            let d = self.documents.get(p).unwrap();
            let mut rank = 0_f64;
            qt.iter().for_each(|t|{

                let tf = calculate_tf(d, t, true);
                let idf = calculate_idf(t, self.documents.len(), &self.df);
                rank += tf * idf;

            });
            if rank > 0.0 {
                Some((p.clone(), rank))
            }else{
                None
            }

        }).collect();

        result.sort_by(|(_, rank1), (_, rank2)| {
            rank2.partial_cmp(rank1).unwrap()

        });

        result

    }
    
    fn search_phrase(&self, query: &[char]) -> Vec<(PathBuf, f64)> {
        let mut result: Vec<(PathBuf, f64)> = Vec::new();
        let qt: Vec<String> = Lexer::new(query).collect();
        if qt.len() == 1 {
            return self.search_singular_exact(query);

        }

        let docs_with_terms: Vec<PathBuf> = match self.docs_with_all_terms(&qt){
            Some(docs) => docs,
            None => return result

        };


        'doc_iter: for d in &docs_with_terms {
            let doc = self.documents.get(d).unwrap();

            for pos in &doc.tf.get(&qt[0]).unwrap().positions {
                if self.contains_tokens_sequential(*pos, qt.clone(), doc){
                    result.push((d.clone(), 1_f64));
                    continue 'doc_iter;

                }
            }

        }
    


        result.sort_by(|(_, rank1), (_, rank2)| {
            rank2.partial_cmp(rank1).unwrap()

        });

        result

    }

    fn contains_tokens_sequential(&self, pos: usize, qt: Vec<String>, doc: &Document) -> bool {
        let mut expected_pos: usize;
        let mut token_idx = 1;
        if pos + 1 >= doc.count {
            return false
        }

        expected_pos = pos + 1;

        loop {
            if token_idx >= qt.len() {
                return false
            }
            if !self.assert_next_token_pos(doc, expected_pos, &qt[token_idx]) {
                return false;
            }


            if token_idx + 1 >= qt.len() {
                break;
            }

            if expected_pos + 1 >= doc.count {
                return false;
            }

            expected_pos += 1;
            token_idx += 1;
        };


        true

    }

    fn assert_next_token_pos(&self, doc: &Document, pos: usize, t: &String) -> bool {
        if let Some(tf) = doc.tf.get(t) {
            tf.positions.contains(&pos)
        }else{
            false
        }

    }

    fn docs_with_all_terms(&self, qt: &[String]) -> Option<Vec<PathBuf>> {
        let result: Vec<PathBuf> = self.documents
            .iter()
            .filter_map(|(p, d)|{
                if qt.iter().all(|t| d.tf.contains_key(t)){
                    Some(p.clone())
                }else{
                    None

                }

            })
        .collect();


        if result.is_empty() {
            None

        }else{
            Some(result)

        }
    }

    fn get_documents(&self) -> Documents {
        self.documents.clone()

    }

    fn delete_removed_files(&mut self) {
        let to_remove: Vec<PathBuf> = self.documents.par_iter().filter_map(|(k, _)|{
            if !k.exists() {
                Some(k.to_path_buf())
            }else{
                None
            }
        }).collect();

        to_remove.iter().for_each(|path|{
            tracing::info!("Deleting removed file {} from index.", path.display());
            self.remove_document(path.to_path_buf());

        });

    }

    fn reset(&mut self) {
        //CorpusModel { documents: Documents::new(), df: DocumentFrequency::new(), df_stemmed: DocumentFrequency::new(), path: None }
        self.documents = Documents::new();
        self.df = DocumentFrequency::new();
        self.df_stemmed = DocumentFrequency::new();
        self.path = None;
    }
    fn add_document_batched(&mut self, batch: Vec<(PathBuf, Vec<char>)>, dir: PathBuf){
        todo!()

    }
}
