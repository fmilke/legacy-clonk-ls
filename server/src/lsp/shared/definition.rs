use tower_lsp::lsp_types::SemanticToken;
use crate::lsp::{highlight_helper::{add_semantic_token, add_semantic_token_at, Context}, token_types::TokenTypes};
use std::{collections::HashMap, str::FromStr};

pub const NODE_KIND_SECTION: &str = "section";
pub const NODE_KIND_SECTION_NAME: &str = "section_name";
pub const NODE_KIND_PROPERTY: &str = "property";
pub const NODE_KIND_IDENTIFIER: &str = "identifier";

pub trait IniDefsProvider {
    fn get_def(section_name: &str, key: &str) -> Option<&'static Definition>;
}

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
                ValueType::extract_semantic_tokens_by_sep(
                    node,
                    ctx,
                    source,
                    ctx.token_types.number,
                );
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

pub type Defs<'a> = HashMap<&'a str, HashMap<&'a str, Definition>>;

#[derive(Debug)]
pub struct Definition {
    pub value_type: ValueType,
    pub description: &'static str,
}

//impl Definition {
//    pub fn get_def<'a>(section_name: &str, key: &str) -> Option<&'a Definition> {
//        let map = &*DEFS;
//        match map.get(section_name) {
//            Some(inner_map) => inner_map.get(key),
//            None => None,
//        }
//    }
//}
//
//

/*
(source_file
     (section
        (section_name
          (identifier))
        (property
          (joined_value))
        (property
          (joined_value))
        (property
          (joined_value))
        (property
          (joined_value)))
*/

pub fn collect_semantic_tokens<T: IniDefsProvider>(
    tree: &tree_sitter::Tree,
    lut: TokenTypes,
    source: &str,
) -> Vec<SemanticToken> {
    let source_bytes = source.as_bytes();

    tracing::trace!("collecting semantic tokens for scenario.txt");

    let mut cursor = tree.walk();

    if !cursor.goto_first_child() || !cursor.goto_first_child() {
        tracing::error!("Expected child, but had none");
        return vec![];
    }

    let mut c = Context {
        token_types: lut,
        ..Context::default()
    };

    let ctx = &mut c;

    let mut section_name = "UNDEFINED_SECTION";

    loop {
        loop {
            let node = cursor.node();

            if !node.is_error() {
                match node.kind() {
                    NODE_KIND_SECTION_NAME => {
                        if let Some(name) = node.child(1) {
                            add_semantic_token(ctx, ctx.token_types.keyword, &name);

                            if let Ok(concrete_section_name) = name.utf8_text(source_bytes) {
                                section_name = concrete_section_name;
                            }
                        }
                    }
                    NODE_KIND_PROPERTY => {
                        if let Some(key) = node.child(0) {
                            add_semantic_token(ctx, ctx.token_types.method, &key);

                            if let Some(operator) = node.child(1) {
                                add_semantic_token(ctx, ctx.token_types.operator, &operator);
                            }

                            if let Some(value) = node.child(2) {
                                if let Ok(concrete_key) = key.utf8_text(source_bytes) {
                                    if let Ok(concrete_value) = value.utf8_text(source_bytes) {
                                        if let Some(def) =
                                            T::get_def(section_name, concrete_key)
                                        {
                                            def.value_type.extract_semantic_tokens(
                                                &value,
                                                ctx,
                                                concrete_value,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        tracing::debug!("Unexpected node kind: {}", node.kind());
                    }
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        if !cursor.goto_parent() || !cursor.goto_next_sibling() || !cursor.goto_first_child() {
            break;
        }
    }

    c.collection
}
