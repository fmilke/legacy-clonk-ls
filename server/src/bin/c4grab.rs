use std::env;

use legacy_clonk_ls::core::{parse::{parse_file, FileId}, signatures::SignatureCollector};
use tokio;

#[tokio::main]
async fn main() {
    let fname = env::args().skip(1).next().expect("Missing argument");

    println!("Filename {}", &fname);

    let file_id = FileId::from_path(&fname).expect("Could not create file id");
    let (tree, content) =  parse_file(&file_id).expect("Could not parse file");

    let sigs = SignatureCollector::collect(file_id, &tree, &content.as_slice()).unwrap();

    let json = serde_json::to_string_pretty(&sigs)
        .expect("Could not make json string");

    println!("Signatures:\n{}", json);
}

