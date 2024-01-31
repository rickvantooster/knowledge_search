use std::{path::{Path, PathBuf}, sync::{Arc, Mutex, RwLock}, fs::read_dir, char};

use rayon::prelude::*;
use notify::{Event, PollWatcher, Config, Watcher};
use rayon::iter::ParallelBridge;

use crate::{model::{GLOB_CORPUS, CorpusModel}, parser::{txt::parse_txt, docx::get_docx_text_manual, html::{get_html_text, get_xhtml}, pdf::get_pdf, error::ParserError}, threadpool::ThreadPool};

pub struct IndexerTask {
    path: PathBuf,
    index_path: PathBuf,

}

impl IndexerTask {
    pub fn new( path: PathBuf, index_path: PathBuf) -> Self {
        IndexerTask { path, index_path }
    }


    pub fn create_watcher(&mut self) -> notify::Result<PollWatcher> {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = PollWatcher::new(tx, Config::default().with_manual_polling())?;
        watcher.watch(self.path.as_ref(), notify::RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        handle_event(&event);
                        GLOB_CORPUS.get().unwrap().write().unwrap().store();
                    },
                    Err(e) => tracing::error!("Watch error {e:?}")
                }

            };

        });

        Ok(watcher)

    }

}

impl Drop for IndexerTask {
    fn drop(&mut self) {
        GLOB_CORPUS.get().unwrap().write().unwrap().store_with_name(&self.index_path);
        println!("Gracefully shutting down Indexer thread");
    }
}

pub fn is_file_supported(file: &Path) -> bool {

    let ext_possible = file.extension();

    if let Some(ext) = ext_possible {
        return match ext.to_str().unwrap_or("") {
            "txt" | "docx" | "html" | "htm" | "xhtml" | "xml" | "pdf" => true,
            _ => false

        }

    }

    false
}

pub fn contents_by_file_type(file: &Path) -> Result<Option<Vec<char>>, ParserError> {

    let ext_possible = file.extension();
    if let Some(ext) = ext_possible {
        let content = match ext.to_str().unwrap_or("") {

            "txt" => parse_txt(file),
            "docx" => get_docx_text_manual(file),
            "html" | "htm" => get_html_text(file),
            "xhtml" | "xml" => get_xhtml(file),
            "pdf" => get_pdf(file),
            _ => {
                tracing::info!("Filepath ({}), File type {} is currently not supported", file.display(), ext.to_str().unwrap());
                return Ok(None);
            }
        }?;

        if !content.is_empty(){
            return Ok(Some(content));
        }
    }

    Ok(None)
    

}
fn handle_event(evn: &Event) {
    if evn.kind.is_create() || evn.kind.is_modify() {
        tracing::info!("event is file creation | file modification");

        evn.paths.iter()
            .filter(|p|{
                let dot_file = p.file_name().and_then(|s| s.to_str()).map(|s|s.starts_with('.')).unwrap_or(false);
                
                let should_reindex = GLOB_CORPUS.get().unwrap().read().unwrap().needs_reindex(p);
                if let Ok(reindex) = should_reindex {
                    return p.is_file() && reindex && !dot_file
                }else if let Err(e) = should_reindex{
                    tracing::error!("{e}");
                }

                false

            })
            .for_each(|p| {
                let result = contents_by_file_type(Path::new(&p));
                if let Ok(Some(content)) = result {
                    let mut model = GLOB_CORPUS.get().unwrap().write().unwrap();
                    model.add_document(p.clone(), &content);
                }else if let Err(err) = result {
                    tracing::error!("{err}");


                }
            });
    }else if let notify::EventKind::Remove(r) = evn.kind {
        if r == notify::event::RemoveKind::File {
            evn.paths.iter()
            .for_each(|p|{
                let dot_file = p.file_name().and_then(|s| s.to_str()).map(|s|s.starts_with('.')).unwrap_or(false);
                if !dot_file {
                    let mut model = GLOB_CORPUS.get().unwrap().write().unwrap();
                    model.remove_document(p.clone());

                }

            });
        }else{
            evn.paths.iter().for_each(|p|{
                let dot_file = p.file_name().and_then(|s| s.to_str()).map(|s|s.starts_with('.')).unwrap_or(false);
                if dot_file {
                    return;

                }
                let mut model = GLOB_CORPUS.get().unwrap().write().unwrap();

                model.get_documents().iter().filter(|(k, _)|{
                    k.starts_with(p.clone())

                }).for_each(|(p, _)|{
                    model.remove_document(p.clone());

                })

            });

        }


    }

}


pub fn add_dir_to_corpus(dir_path: &Path) -> Result<(), ()> {
    //println!("processing dir: {}", dir_path.display());

    let dir = read_dir(dir_path).map_err(|e| {
         tracing::error!("could not open directory {} for indexing: {e}",
         dir_path.display());
    })?;

    let contents: Vec<(PathBuf, Vec<char>)> = dir.par_bridge().filter_map(|file|{
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Could not read next file in directory {} for indexing: {e}",
                    dir_path.display());
                return None;

            },
        };

        let file_path = file.path();


        let dot_file = file.path().file_name().and_then(|s| s.to_str()).map(|s|s.starts_with('.')).unwrap_or(false);

        if dot_file {
            tracing::info!("Skipping dotfile {}", file.path().display());
            return None;
        }

        match file.file_type() {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Could not parse file type of file {}: {e}", file_path.display());
                return None;
            }
        }


        if file_path.is_dir() {
            let _ = add_dir_to_corpus(&file_path);
            return None;
        }

        let possible_reindex = GLOB_CORPUS.get().unwrap().read().unwrap().needs_reindex(&file_path);

        if let Err(e) = possible_reindex {
            tracing::error!("Could not check if file {} needed reindexing. Error {e}", file_path.display());
            return None;
        }

        let reindex = possible_reindex.unwrap();

        if !reindex {
            return None;

        }

        let result = contents_by_file_type(Path::new(&file_path));
        if let Err(ref err) = result {
            tracing::error!("{err}");
            return None;
        }

        if let Ok(Some(content)) = result {
            return Some((file_path, content));
        }


        None


    }).collect();

    contents.par_iter().for_each(|(file_path, content)|{
        //println!("adding file {} to corpus: ", file_path.display());
        GLOB_CORPUS.get().unwrap().write().unwrap().add_document(file_path.clone(), &content);

    });


    Ok(())
}



pub fn add_dir_to_corpus_joined(dir_path: &Path) -> Result<(), ()> {
    //println!("processing dir: {}", dir_path.display());

    let dir = read_dir(dir_path).map_err(|e| {
         tracing::error!("could not open directory {} for indexing: {e}",
         dir_path.display());
    })?;

    dir.par_bridge().filter_map(|file|{
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Could not read next file in directory {} for indexing: {e}",
                    dir_path.display());
                return None;

            },
        };

        let file_path = file.path();


        let dot_file = file.path().file_name().and_then(|s| s.to_str()).map(|s|s.starts_with('.')).unwrap_or(false);

        if dot_file {
            tracing::info!("Skipping dotfile {}", file.path().display());
            return None;
        }

        match file.file_type() {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Could not parse file type of file {}: {e}", file_path.display());
                return None;
            }
        }


        if file_path.is_dir() {
            let _ = add_dir_to_corpus_joined(&file_path);
            return None;
        }

        let possible_reindex = GLOB_CORPUS.get().unwrap().read().unwrap().needs_reindex(&file_path);

        if let Err(e) = possible_reindex {
            tracing::error!("Could not check if file {} needed reindexing. Error {e}", file_path.display());
            return None;
        }

        let reindex = possible_reindex.unwrap();

        if !reindex {
            return None;

        }

        let result = contents_by_file_type(Path::new(&file_path));
        if let Err(ref err) = result {
            tracing::error!("{err}");
            return None;
        }

        if let Ok(Some(content)) = result {
            return Some((file_path, content));
        }


        None


    }).for_each(|(file_path, content)|{
        GLOB_CORPUS.get().unwrap().write().unwrap().add_document(file_path.clone(), &content);
    });


    Ok(())
}


