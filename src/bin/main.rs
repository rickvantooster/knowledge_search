use std::{path::{Path, PathBuf},  fs::File, sync::{Arc, RwLock}, ops::Deref, hint::black_box};
use knowledge_search::{indexer::add_dir_to_corpus_joined, model::{base::Model, invertedmodel::InvertedModel, json_model::JsonModel, path_to_index_name, CorpusModel, ModelType}};
use tracing_subscriber::{layer::SubscriberExt, Layer, util::SubscriberInitExt, filter::LevelFilter};
use knowledge_search::tui::tui;


use knowledge_search::{indexer::IndexerTask, model::GLOB_CORPUS};

fn init_logging() {
    let file = File::create("knowledge_search.log").unwrap();
    let background_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(background_log.with_filter(LevelFilter::INFO)
      ).init();

}



fn setup_glob_corpus(path: PathBuf){
    let index_path = path_to_index_name(&path, ModelType::Inverted);
    

    //let corpus = Arc::new(RwLock::new(JsonModel::from_disk(&index_path).unwrap_or(JsonModel::new())));
    let corpus = Arc::new(RwLock::new(CorpusModel::new_inverted_model(&index_path)));
    GLOB_CORPUS.set(corpus).unwrap();
    let start = std::time::Instant::now();

    let _ = add_dir_to_corpus_joined(&path);
    GLOB_CORPUS.get().unwrap().write().unwrap().delete_removed_files();
    let end = std::time::Instant::now();
    println!("indexing {} took {}ms", path.display(), end.duration_since(start).as_millis());

    GLOB_CORPUS.get().unwrap().write().unwrap().store_with_name(&PathBuf::from(index_path.deref()));

}

fn bench_alloc(path: PathBuf) {
    {
        //let model = JsonModel::from_disk(&path_to_index_name(&path, ModelType::Json)).unwrap_or(JsonModel::new());
        let model = CorpusModel::new_inverted_model(&path);
        let corpus = Arc::new(RwLock::new(model));
        GLOB_CORPUS.set(corpus).unwrap();
    }
    let _ = black_box(add_dir_to_corpus_joined(&path));

    GLOB_CORPUS.get().unwrap().write().unwrap().store_with_name(&path_to_index_name(&path, ModelType::Inverted));
    let query: Vec<char> = "Tesla".chars().collect();
    let results: Vec<(PathBuf, f64)> = GLOB_CORPUS.get().unwrap().read().unwrap().search_simple(&query).iter().cloned().collect();
    println!("results: {:#?}", results);
}

fn entry(path: PathBuf) {
    //bench_indexing(path.to_path_buf());
    //bench_alloc(path);
    
    let index_path = path_to_index_name(&path, ModelType::Inverted);
    setup_glob_corpus(path.to_path_buf());
    let mut indexer = IndexerTask::new(path.clone(), index_path);

    tui(&mut indexer).unwrap();
}


fn main() {

    //rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    init_logging();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: <{}> <DIRECTORY>", args.get(0).unwrap());
        std::process::exit(1);
    }

    let cwd = std::env::current_dir().unwrap();
    let possible_dir = cwd.join(PathBuf::from(args.get(1).unwrap()));

    if !possible_dir.exists() || !possible_dir.is_dir(){
        eprintln!("Error: {} is not a directory", args.get(1).unwrap());
        std::process::exit(2);
    }

    //let mut possible_dir = PathBuf::from("./20_newsgroups");
    entry(possible_dir);
}

