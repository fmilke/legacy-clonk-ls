use super::node_kind::NODE_KIND_SECTION_NAME;
use crate::lsp::{
    highlight_helper::{add_semantic_token, Context},
    scenario_txt_handler::{
        definition::{Definition},
        node_kind::NODE_KIND_PROPERTY,
    },
    token_types::TokenTypes,
};
use tower_lsp::lsp_types::SemanticToken;

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

pub fn collect_semantic_tokens(
    tree: &tree_sitter::Tree,
    lut: TokenTypes,
    source: &str,
) -> Vec<SemanticToken> {
    let source_bytes = source.as_bytes();

    tracing::info!("collecting semantic tokens for scenario.txt");

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
                                        if let Some(def) = Definition::get_def(section_name, concrete_key) {
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
