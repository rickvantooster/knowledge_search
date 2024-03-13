/*
use std::{sync::{Arc, Mutex, RwLock}, path::Path, fs::read_dir, io::{BufReader, Read}};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knowledge_search::{model::{JsonModel, GLOB_CORPUS}, indexer::{scan_dir_par, add_dir_to_corpus_parallel, add_dir_to_corpus_parallel_joined_iter, add_dir_to_corpus_parallel_merge}};
//use knowledge_search::{model::CorpusModel, indexer::contents_by_file_type, lexer::{StrLexer, Lexer}};


pub fn criterion_benchmark(c: &mut Criterion) {
    let source = Path::new("20_newsgroups");
    let corpus = Arc::new(RwLock::new(JsonModel::new()));
    GLOB_CORPUS.set(corpus).unwrap();


    c.bench_function("scan_dir_par", |b| b.iter(||{
        let local_corpus = Arc::new(JsonModel::new());
        let _ = black_box(scan_dir_par(source, local_corpus));
    }));

    c.bench_function("add_dir_par", |b| b.iter(||{
        {
            GLOB_CORPUS.get().unwrap().write().unwrap().reset();
        }
        let _ = black_box(add_dir_to_corpus_parallel(source));
    }));

    c.bench_function("add_dir_par_joined_iter", |b| b.iter(||{
        {
            GLOB_CORPUS.get().unwrap().write().unwrap().reset();
        }
        let _ = black_box(add_dir_to_corpus_parallel_joined_iter(source));
    }));

    c.bench_function("add_dir_par_merged", |b| b.iter(||{
        {
            GLOB_CORPUS.get().unwrap().write().unwrap().reset();
        }
        let _ = black_box(add_dir_to_corpus_parallel_merge(source));
    }));
}
criterion_group! {name = benches; config = Criterion::default().sample_size(10); targets = criterion_benchmark}
criterion_main!(benches);
*/
