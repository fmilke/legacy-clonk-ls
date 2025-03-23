use crate::lang::Translation;

use super::{asset_handler::AssetHandler, doc::Document, token_types::TokenTypes};
use definition::Definition;
use node_kind::{NODE_KIND_PROPERTY, NODE_KIND_SECTION, NODE_KIND_SECTION_NAME};
use tower_lsp::lsp_types::SemanticToken;

mod definition;
mod highlighting;
mod node_kind;

#[derive(Debug, Clone, Default)]
pub struct ScenarioTxtHandler;

impl AssetHandler for ScenarioTxtHandler {
    fn collect_semantic_tokens(
        &self,
        tree: &tree_sitter::Tree,
        lut: TokenTypes,
        source: &str,
    ) -> Vec<SemanticToken> {
        highlighting::collect_semantic_tokens(tree, lut, source)
    }

    fn get_hover_text(
        &self,
        doc: &Document,
        pos: tower_lsp::lsp_types::Position,
    ) -> Option<String> {
        let mut cursor = doc.tree.walk();
        let point = Document::point_to_pos(pos);
        let mut section_name: Option<&str> = None;

        loop {
            let node = cursor.node();

            match node.kind() {
                NODE_KIND_PROPERTY => {
                    if let Some(key) = node.child(0) {
                        // TODO: check if hover is on this child

                        let text = match key.utf8_text(doc.source.as_bytes()) {
                            Ok(s) => s,
                            _ => return None,
                        };

                        if let Some(ref section_name) = section_name {
                            tracing::info!("Got section {}", section_name);
                            if let Some(def) = Definition::get_def(section_name, text) {
                                if let Some(s) = Translation::get_translation(def.description) {
                                    return Some(s.to_owned());
                                }
                            }
                        }
                    }
                }
                NODE_KIND_SECTION => {
                    if let Some(first_child) = node.child(0) {
                        if first_child.kind() == NODE_KIND_SECTION_NAME {
                            if let Some(name) = first_child.child(1) {
                                if let Ok(concrete_section_name) =
                                    name.utf8_text(doc.source.as_bytes())
                                {
                                    section_name = Some(concrete_section_name);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if cursor.goto_first_child_for_point(point).is_none() {
                break;
            }
        }

        None
    }
}
