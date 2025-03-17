use tower_lsp::lsp_types::SemanticToken;
use super::{asset_handler::AssetHandler, highlighting::Highlighter, token_types::TokenTypes};

#[derive(Debug, Clone, Default)]
pub struct ScriptHandler;

impl AssetHandler for ScriptHandler {
    fn collect_semantic_tokens(&self, tree: &tree_sitter::Tree, lut: TokenTypes, _: &str) -> Vec<SemanticToken> {
        Highlighter::collect_tokens(tree, lut)
    }
}
