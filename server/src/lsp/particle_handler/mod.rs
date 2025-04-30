use super::{
    asset_handler::AssetHandler,
    scenario_txt_handler::definition::parse_defs,
    shared::definition::{collect_semantic_tokens, C4Ini, Definition, Defs, IniDefsProvider},
    token_types::TokenTypes,
};
use lazy_static::lazy_static;
use tower_lsp::lsp_types::SemanticToken;

const UNPARSED_DEFS: &str = include_str!("defs.csv");

lazy_static! {
    static ref DEFS: Defs<'static> = init_definitions();
}

pub struct ParticleDefs;

impl IniDefsProvider for ParticleDefs {
    fn get_def(section_name: &str, key: &str) -> Option<&'static Definition> {
        let map = &*DEFS;
        match map.get(section_name) {
            Some(inner_map) => inner_map.get(key),
            None => None,
        }
    }
}

fn init_definitions() -> Defs<'static> {
    parse_defs(UNPARSED_DEFS, |ctx| {
        ctx.add();
    })
}

#[derive(Debug, Clone, Default)]
pub struct ParticleHandler;

impl AssetHandler for ParticleHandler {
    fn collect_semantic_tokens(
        &self,
        tree: &tree_sitter::Tree,
        lut: TokenTypes,
        source: &str,
    ) -> Vec<SemanticToken> {
        collect_semantic_tokens::<ParticleDefs>(tree, lut, source)
    }

    fn get_hover_text(&self, doc: &super::doc::Document, pos: tower_lsp::lsp_types::Position) -> Option<String> {
        C4Ini::get_hover_text::<ParticleDefs>(doc, pos)
    }
}
