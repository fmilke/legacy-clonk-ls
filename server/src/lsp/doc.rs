use tower_lsp::lsp_types::{Position, Url};
use tracing::info;
use tree_sitter::{Point, Tree};

use crate::core::kind::NODE_KIND_FN_DEF;

pub enum QueryableItem {
    Function(String),
    Constant(String),
    Unused,
}

#[derive(Debug)]
pub struct Document {
    #[allow(dead_code)]
    pub url: Url,
    pub tree: Tree,
    pub source: String,
}

impl Document {
    pub fn new(url: Url, tree: Tree, source: String) -> Self {
        Document { url, tree, source, }
    }

    pub fn get_item_at_pos(&self, pos: Position) -> Option<QueryableItem> {

        let mut cursor = self.tree.walk();
        let point = Document::point_to_pos(pos);

        let mut child = cursor.goto_first_child_for_point(point);

        while let Some(c) = cursor.goto_first_child_for_point(point) {
            child = Some(c);
        }

        if let Some(_) = child {
            let node = cursor.node();

            let text = match node.utf8_text(self.source.as_bytes()) {
                Ok(s) => {
                    s
                },
                _ => return None,
            };

            let current_node_kind = node.kind();
            cursor.goto_parent();
            let parent_node_kind = cursor.node().kind();

            info!("parent node kind: {}, node kind: {}", parent_node_kind, current_node_kind);

            if parent_node_kind == NODE_KIND_FN_DEF {
                info!("found function definition; function name: {}", text);
                Some(QueryableItem::Function(String::from(text)))
            } else {
                // we assume, that any other context, in which 
                // an identifier occurs, must be an expression
                // and we can check for constants.
                // if this assumption is wrong, we need a way to check,
                // if we are inside an expression

                if current_node_kind == "identifier" {
                    Some(QueryableItem::Constant(String::from(text)))
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn point_to_pos(pos: Position) -> Point {
        Point {
            row: pos.line as usize,
            column: pos.character as usize,
        }
    }
}

