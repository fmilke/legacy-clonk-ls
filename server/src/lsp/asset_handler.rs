use tower_lsp::lsp_types::SemanticToken;
use super::{doc::Document, token_types::TokenTypes};

pub trait AssetHandler {
    fn collect_semantic_tokens(&self, tree: &tree_sitter::Tree, lut: TokenTypes, source: &str) -> Vec<SemanticToken>;
    fn get_hover_text(&self, _: &Document, _: tower_lsp::lsp_types::Position) -> Option<String> {
        None
    }
}
