use super::{asset_handler::AssetHandler, doc::Document, token_types::TokenTypes};
use definition::Definition;
use node_kind::NODE_KIND_PROPERTY;
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

        loop {
            let node = cursor.node();

            tracing::info!("hover: Node kind: {}", node.kind());
            match node.kind() {
                NODE_KIND_PROPERTY => {
                    if let Some(key) = node.child(0) {
                        // TODO: check if hover is on this child

                        let text = match key.utf8_text(doc.source.as_bytes()) {
                            Ok(s) => s,
                            _ => return None,
                        };

                        tracing::info!("hover: text: {}", text);
                        return Definition::get_def("Game", text)
                            .map(|def| def.description.to_string());
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
