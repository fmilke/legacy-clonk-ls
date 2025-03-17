
use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::lsp::highlight_helper::{add_semantic_token, add_semantic_token_at, Context};

#[derive(Debug, Clone)]
pub enum ValueType {
    Num,
    NumList,
    IdList,
    Id,
    String,
    Unknown,
}

impl ValueType {
    pub fn extract_semantic_tokens(&self, node: &tree_sitter::Node, ctx: &mut Context, source: &str) {
        match self {
            ValueType::Num => {
                add_semantic_token(ctx, ctx.token_types.number, &node);
            },
            ValueType::IdList => {
                let mut start = node.start_position();
                let mut end = node.start_position();
                tracing::info!("id liiiiist - start: {}", start.column);

                for pair in source.split(';') {
                    let original_start = start.column;
                    tracing::info!("original_start: {}", start.column); 
                    if let Some((key, value))  = pair.split_once('=') {
                        end.column = start.column + key.len();
                        add_semantic_token_at(ctx, ctx.token_types.id, start, end);
                        tracing::info!("key ({}): start: {} - end {}", key, start.column, end.column);

                        start.column += key.len();
                        end.column = start.column + 1;
                        add_semantic_token_at(ctx, ctx.token_types.operator, start, end);
                        tracing::info!("operator: start: {} - end {}", start.column, end.column);

                        start.column = end.column;
                        end.column += value.len();
                        add_semantic_token_at(ctx, ctx.token_types.number, start, end);
                        tracing::info!("value: start: {} - end {}", start.column, end.column);
                    }

                    start.column = original_start + 1 + pair.len();
                }
            },
            _ => {},
        }
    }
}

pub fn get_value_type(key_name: impl AsRef<str>) -> ValueType {

    if let Some(def) = Definition::get_def("Game", key_name.as_ref()) {
        return def.value_type.clone();
    }

    match key_name.as_ref() {
        "Icon" => ValueType::Num,
        "Goals" => ValueType::IdList,
        "Wealth" => ValueType::Num,
        "MaxPlayer" => ValueType::Num,
        _ => ValueType::Unknown,
    }
}

type Defs<'a> = HashMap<&'a str, HashMap<&'a str, Definition>>;

lazy_static! {
    static ref DEFS: Defs<'static> = init_definitions();
}


pub struct Definition {
    pub value_type: ValueType,
    pub description: &'static str,
}

impl Definition {
    pub fn get_def<'a>(section_name: &str, key: &str) -> Option<&'a Definition> {
        let map = &*DEFS;
        match map.get(section_name) {
            Some(inner_map) => {
                inner_map.get(key)
            },
            None => None,
        }
    }
}

fn init_definitions<'a>() -> HashMap<&'a str, HashMap<&'a str, Definition>> {
    let mut map = HashMap::new();

    let mut game_section = HashMap::new();
    game_section.insert("Rules", Definition {
        value_type: ValueType::IdList,
        description: "This is description for definition",
    });

    map.insert("Game", game_section);

    map
}
