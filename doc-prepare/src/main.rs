use std::{fs::File, io::Read, path::PathBuf};
use anyhow::Ok;
use serde::Deserialize;

const FN_DIR: &str = "../../lcdocs/sdk/script/fn/";

fn main() {
    parse_fn_defs().unwrap();
}

fn parse_fn_defs() -> anyhow::Result<()> {

    let dir = std::fs::read_dir(FN_DIR)?;

    for e in dir {
        match e {
            Result::Ok(entry) => {
                let _ = parse_fn_file(&entry.path());
            },
            _ => {
            },
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct DocFnFile {
    //pub funcs: Vec<DocFn>
}

#[derive(Debug, Deserialize)]
pub struct DocFn {
    pub title: String,
    pub desc: String,
    pub syntax: DocFnSyntax,
}

#[derive(Debug, Deserialize)]
pub struct DocFnSyntax {
    #[serde(rename(deserialize = "dataType"))]
    pub return_type: String,
    pub params: DocFnParams, 
}

#[derive(Debug, Deserialize)]
pub struct DocFnParams {
    pub param: Vec<DocFnParam>,
}

#[derive(Debug, Deserialize)]
pub struct DocFnParam {
    #[serde(rename(deserialize = "type"))]
    pub data_type: String,
    pub name: String,
    #[serde(rename(deserialize = "desc"))]
    pub description: String,
}

fn parse_fn_file(path: &PathBuf) -> anyhow::Result<DocFnFile> {

    let mut s = String::new();
    let mut f = File::open(path)?;
    f.read_to_string(&mut s)?;
    let document = serde_xml::from_str::<DocFnFile>(s.as_str())?;

    Ok(document)
}
