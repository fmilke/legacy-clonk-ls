use std::env;

use legacy_clonk_ls::core::{parse::{parse_file, FileId}, signatures::SignatureCollector};
use tokio;

#[tokio::main]
async fn main() {

    // TODO: Properly configure logging
    /*
    if let Ok(file) = std::fs::File::create("/home/fmi/lsp-log") {
        let _ = WriteLogger::init(log::LevelFilter::Trace, simplelog::Config::default(), file);
    }
    */

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let fname = env::args().skip(1).next().expect("Missing argument");

    println!("Filename {}", &fname);

    let file_id = FileId::from_path(&fname).expect("Could not create file id");
    let (tree, content) =  parse_file(&file_id).expect("Could not parse file");

    let sigs = SignatureCollector::collect(file_id, &tree, &content.as_slice()).unwrap();

    let json = serde_json::to_string_pretty(&sigs)
        .expect("Could not make json string");

    println!("Signatures:\n{}", json);
}

