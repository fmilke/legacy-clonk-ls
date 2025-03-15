use std::{fs, path::PathBuf, str::FromStr};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use tree_sitter::{Parser, Tree};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileId {
    pub path: Box<PathBuf>,
}

impl FileId {
    pub fn from_path(s: &String) -> anyhow::Result<Self> {
        Ok(FileId {
            path: Box::new(PathBuf::from_str(s)?),
        })
    }
}

pub fn parse_file(file_id: &FileId) -> anyhow::Result<(Tree, Vec<u8>)> {

    // TODO: remove this clone
    let content = fs::read(*file_id.path.clone())?;
    let mut parser = Parser::new();

    parser
        .set_language(tree_sitter_c4script::language())?;

    parser
        .parse(&content, None)
        .context("Could not parse file")
        .map(|v| (v, content))
}

