use anyhow::bail;
use serde::{Deserialize, Serialize};
use tree_sitter::Tree;
use crate::core::kind::NODE_KIND_SOURCE_FILE;

use super::{kind::NODE_KIND_FN_DEF, parse::FileId};

const DEBUG_WALK: bool = true;

pub struct SignatureCollector;

#[derive(Debug, Deserialize)]
pub enum C4DataType {
    #[serde(rename(deserialize = "int"))]
    Int,
    #[serde(rename(deserialize = "id"))]
    Id,
    #[serde(rename(deserialize = "bool"))]
    Bool,
    #[serde(rename(deserialize = "string"))]
    String,
    #[serde(rename(deserialize = "object"))]
    Object,
    #[serde(rename(deserialize = "array"))]
    Array,
    #[serde(rename(deserialize = "any"))]
    Any,
}

impl C4DataType {
    pub fn moniker(&self) -> &'static str {
        match self {
            C4DataType::Int => "int",
            C4DataType::Id => "id",
            C4DataType::Bool => "bool",
            C4DataType::Any => "any",
            C4DataType::Array => "array",
            C4DataType::Object => "object",
            C4DataType::String => "string",
        }
    }
}

impl std::fmt::Display for C4DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Param {
    pub name: String,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug)]
pub struct Signature {
    pub name: String,
    pub params: Vec<Param>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSignatures {
    pub file_id: FileId,
    pub signatures: Vec<Signature>,
}

impl SignatureCollector {
    pub fn collect(file_id: FileId, tree: &Tree, source: impl AsRef<[u8]>) -> anyhow::Result<FileSignatures> {

        let source = source.as_ref();

        let mut sigs = FileSignatures {
            file_id,
            signatures: vec![],
        };

        let mut cursor = tree.walk();
        if cursor.node().kind() != NODE_KIND_SOURCE_FILE {
            bail!("Expected node with kind '{}'", NODE_KIND_SOURCE_FILE);
        }

        if !cursor.goto_first_child() {
            bail!("Expected node '{}' to have ", NODE_KIND_SOURCE_FILE);
        }

        loop {
            let node = cursor.node();

            if DEBUG_WALK {
                println!("Current node: {}", node.kind());
            }

            match node.kind() {
                NODE_KIND_FN_DEF => {
                    let mut sig = Signature::default();

                    if let Some(ref name) = node.child_by_field_name("name") {
                        match name.utf8_text(source) {
                            Ok(n) => sig.name = n.to_string(),
                            Err(e) => {
                                if DEBUG_WALK {
                                    println!("Could not parse name of function signature: {}", e);
                                }
                                continue;
                            },
                        }
                    } else {
                        // should never be reached, but without
                        // a function name, there is nothing 
                        // sound we can do here
                        continue;
                    }
                    // enter children of function_definition
                    if cursor.goto_first_child() {

                        loop {
                            if cursor.node().kind() == "parameter_list" {
                                let params = cursor.node();
                                if DEBUG_WALK {
                                    println!("Found parameter_list with {} children", params.child_count());
                                }

                                // enter children of parameter_list
                                if cursor.goto_first_child() {
                                    loop {
                                        let child = cursor.node();
                                        if child.kind() == "param" {
                                            match child.child_count() {
                                                1 => {
                                                    let name_node = child.child(0).unwrap();
                                                    let name = name_node.utf8_text(source).unwrap();
                                                    let p = Param {
                                                        name: name.to_string(),
                                                    };

                                                    sig.params.push(p);
                                                },
                                                2 => {
                                                    let name_node = child.child(1).unwrap();
                                                    let name = name_node.utf8_text(source).unwrap();
                                                    let p = Param {
                                                        name: name.to_string(),
                                                    };

                                                    sig.params.push(p);
                                                },
                                                c => {
                                                    println!("Unexpected child count of param node: {}", c);
                                                },
                                            }
                                        }

                                        if !cursor.goto_next_sibling() {
                                            break;
                                        }
                                    }

                                    // leave children of function_definition
                                    cursor.goto_parent();
                                }
                            }

                            if !cursor.goto_next_sibling() {
                                break;
                            }
                        }

                        // leave children of function_definition
                        cursor.goto_parent();
                        sigs.signatures.push(sig);
                    }
                },
                _ => {},
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        Ok(sigs)
    }
}

