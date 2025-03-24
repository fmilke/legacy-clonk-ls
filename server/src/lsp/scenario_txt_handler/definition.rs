use std::{collections::HashMap, str::FromStr};
use lazy_static::lazy_static;
use crate::lsp::highlight_helper::{add_semantic_token, add_semantic_token_at, Context};

#[derive(Debug, Clone)]
pub enum ValueType {
    Integer,
    DWORD,
    IntegerList,
    IdList,
    MatList,
    Id,
    String,
    Unknown,
}

impl ValueType {
    fn extract_semantic_tokens_by_sep(
        node: &tree_sitter::Node,
        ctx: &mut Context,
        source: &str,
        token_type: u32,
    ) {
        let mut start = node.start_position();
        let mut end = node.start_position();

        for token in source.split(',') {
            let original_start = start.column;
            end.column = start.column + token.len();
            add_semantic_token_at(ctx, token_type, start, end);
            start.column = original_start + 1 + token.len();
        }
    }

    pub fn extract_semantic_tokens(
        &self,
        node: &tree_sitter::Node,
        ctx: &mut Context,
        source: &str,
    ) {
        match self {
            ValueType::String => {
                add_semantic_token(ctx, ctx.token_types.string, &node);
            }
            ValueType::MatList => {
                // TODO: same as IdList, make reusable
                let mut start = node.start_position();
                let mut end = node.start_position();

                for pair in source.split(';') {
                    let original_start = start.column;
                    if let Some((key, value)) = pair.split_once('=') {
                        end.column = start.column + key.len();
                        add_semantic_token_at(ctx, ctx.token_types.string, start, end);

                        start.column += key.len();
                        end.column = start.column + 1;
                        add_semantic_token_at(ctx, ctx.token_types.operator, start, end);

                        start.column = end.column;
                        end.column += value.len();
                        add_semantic_token_at(ctx, ctx.token_types.number, start, end);
                    }

                    start.column = original_start + 1 + pair.len();
                }
            }
            ValueType::Integer | ValueType::IntegerList | ValueType::DWORD => {
                ValueType::extract_semantic_tokens_by_sep(node, ctx, source, ctx.token_types.number);
            }
            ValueType::IdList => {
                let mut start = node.start_position();
                let mut end = node.start_position();

                for pair in source.split(';') {
                    let original_start = start.column;
                    if let Some((key, value)) = pair.split_once('=') {
                        end.column = start.column + key.len();
                        add_semantic_token_at(ctx, ctx.token_types.id, start, end);

                        start.column += key.len();
                        end.column = start.column + 1;
                        add_semantic_token_at(ctx, ctx.token_types.operator, start, end);

                        start.column = end.column;
                        end.column += value.len();
                        add_semantic_token_at(ctx, ctx.token_types.number, start, end);
                    }

                    start.column = original_start + 1 + pair.len();
                }
            }
            _ => {}
        }
    }
}

impl FromStr for ValueType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IdList" => Ok(ValueType::IdList),
            "Id" => Ok(ValueType::Id),
            "MatList" => Ok(ValueType::MatList),
            "Integer" => Ok(ValueType::Integer),
            "String" => Ok(ValueType::String),
            "DWORD" => Ok(ValueType::DWORD),
            _ => {
                tracing::info!("missing FromStr implentation ValueType for {}", s);
                Ok(ValueType::String)
            }
        }
    }
}

type Defs<'a> = HashMap<&'a str, HashMap<&'a str, Definition>>;

lazy_static! {
    static ref DEFS: Defs<'static> = init_definitions();
}

#[derive(Debug)]
pub struct Definition {
    pub value_type: ValueType,
    pub description: &'static str,
}

impl Definition {
    pub fn get_def<'a>(section_name: &str, key: &str) -> Option<&'a Definition> {
        let map = &*DEFS;
        match map.get(section_name) {
            Some(inner_map) => inner_map.get(key),
            None => None,
        }
    }
}

const UNPARSED_DEFS: &str = include_str!("./scenario_txt_defs.csv");

fn init_definitions<'a>() -> Defs<'a> {
    let mut map: Defs = HashMap::new();

    for line in UNPARSED_DEFS.lines() {
        let mut parts = line.split('|');
        let section_name = parts.next().expect("Getting Scenario.txt section");
        let key_name = parts.next().expect("Getting Scenario.txt key");
        let value_type = parts
            .next()
            .map(|v| ValueType::from_str(v).unwrap())
            .expect("Getting Scenario.txt value type");

        let translation_key = parts.next().expect("Getting Scenario.txt translation key");

        fn add_def(
            map: &mut Defs,
            section_name: &'static str,
            key_name: &'static str,
            value_type: ValueType,
            translation_key: &'static str,
        ) {
            let def = Definition {
                description: translation_key,
                value_type: value_type.clone(),
            };

            tracing::info!(
                "parsed this def {:?} for section {} and property {}",
                &def,
                &section_name,
                &key_name
            );

            match map.get_mut(section_name) {
                Some(sub_map) => {
                    sub_map.insert(key_name, def);
                }
                None => {
                    let mut sub_map = HashMap::new();
                    sub_map.insert(key_name, def);
                    map.insert(section_name, sub_map);
                }
            }
        }

        if section_name == "Player" {
            add_def(
                &mut map,
                "Player1",
                key_name,
                value_type.clone(),
                translation_key,
            );
            add_def(
                &mut map,
                "Player2",
                key_name,
                value_type.clone(),
                translation_key,
            );
            add_def(
                &mut map,
                "Player3",
                key_name,
                value_type.clone(),
                translation_key,
            );
            add_def(&mut map, "Player4", key_name, value_type, translation_key);
        } else {
            add_def(
                &mut map,
                section_name,
                key_name,
                value_type,
                translation_key,
            );
        }
    }

    map
}
