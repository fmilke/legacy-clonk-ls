use super::{asset_handler::AssetHandler, doc::Document, shared::definition::{collect_semantic_tokens, C4Ini}, token_types::TokenTypes};
use definition::ScenarioTxtDefs;
use tower_lsp::lsp_types::SemanticToken;

pub mod definition;
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
        collect_semantic_tokens::<ScenarioTxtDefs>(tree, lut, source)
    }

    fn get_hover_text(
        &self,
        doc: &Document,
        pos: tower_lsp::lsp_types::Position,
    ) -> Option<String> {
        C4Ini::get_hover_text::<ScenarioTxtDefs>(doc, pos)
    }
}
