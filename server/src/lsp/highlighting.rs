use log::debug;
use tower_lsp::lsp_types::SemanticToken; use tree_sitter::{Tree, TreeCursor, Node};
use crate::lsp::token_types::TokenTypes;

#[derive(Debug, Default)]
struct Context {
    collection: Vec<SemanticToken>,
    last_line: u32,
    last_start: u32,
    token_types: TokenTypes,
}

pub struct Highlighter;

impl Highlighter {

    pub fn collect_tokens(tree: &Tree, lut: TokenTypes) -> Vec<SemanticToken> {
        let mut cursor = tree.walk();
        let mut ctx = Context {
            token_types: lut,
            ..Context::default()
        };

        Self::collect_tokens_step(&mut cursor, &mut ctx);
        cursor.goto_first_child();

        ctx.collection
    }

    fn add_token(
        ctx: &mut Context,
        token_type: u32,
        node: &Node,
    ) {
        debug!("Tokenizing node of type: {}", node.kind());

        let start = node.start_position();
        let mut start_row = start.row as u32;
        let start_col = start.column as u32;
        let on_same_line = ctx.last_line == start_row;

        let end = node.end_position();
        let end_row = end.row as u32;
        let end_col = end.column as u32;
        let multiline = start_row != end_row;

        if multiline {
            // TODO: If there are editors not happy with sending a token of the aribitraty length 2000
            // we need to properly pre-calculate the line lengths before

            // First line
            {
                let (delta_line, delta_start) = if on_same_line {
                    (0, start_col - ctx.last_start)
                } else {
                    (start_row - ctx.last_line, start_col)
                };

                ctx.collection.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: 2000,
                    token_type,
                    ..Default::default()
                });
            }

            // Intermediate lines
            loop {
                // Make sure, we are not already on the last line
                start_row += 1;
                if start_row == end_row {
                    break;
                }

                ctx.collection.push(SemanticToken {
                    delta_line: 1,
                    delta_start: 0,
                    length: 2000,
                    token_type,
                    ..Default::default()
                });
            }

            // Last line
            ctx.collection.push(SemanticToken {
                delta_line: 1,
                delta_start: 0,
                length: end_col,
                token_type,
                ..Default::default()
            });

            ctx.last_line = start_row;
            ctx.last_start = 0;

        } else {
            debug!("Start: {}, {}", start_row, start_col);
            debug!("ctx: {}, {}", ctx.last_line, ctx.last_start);

            let (delta_line, delta_start) = if on_same_line {
                (0, start_col - ctx.last_start)
            } else {
                (start_row - ctx.last_line, start_col)
            };

            if on_same_line {
                ctx.last_start += delta_start;
            } else {
                ctx.last_line = start_row;
                ctx.last_start = delta_start;
            }

            ctx.collection.push(SemanticToken {
                delta_line,
                delta_start,
                length: (node.end_position().column - node.start_position().column) as u32,
                token_type,
                ..Default::default()
            });
        }

        
    }

    fn collect_tokens_step(
        cursor: &mut TreeCursor,
        ctx: &mut Context,
    ) {
        loop {
            let node = cursor.node();
            let mut traverse_children = true;

            if node.is_named() {
                match node.kind() {
                    "comment" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.comment,
                            &node,
                        );
                    },

                    "var_definition" => {
                        if let Some(ref child) = node.child(0) {
                            Self::add_token(ctx, ctx.token_types.var_scope, child);
                        }

                        if let Some(ref child) = node.child_by_field_name("const") {
                            Self::add_token(ctx, ctx.token_types.keyword, child);
                        }
                    },
                    "function_definition" => {
                        let mut idx_of_func = 0;

                        if let Some(vis) = node.child_by_field_name("visibility") {
                            Self::add_token(
                                ctx,
                                ctx.token_types.keyword,
                                &vis,
                            );

                            idx_of_func += 1;
                        }

                        if let Some(name) = node.child(idx_of_func) {
                            Self::add_token(
                                ctx,
                                ctx.token_types.keyword,
                                &name,
                            );
                        }

                        if let Some(name) = node.child_by_field_name("name") {
                            Self::add_token(
                                ctx,
                                ctx.token_types.method,
                                &name,
                            );  
                        }
                    },
                    "method_call" => {

                        if let Some(child) = node.child_by_field_name("id") {
                            Self::add_token(
                                ctx,
                                ctx.token_types.id,
                                &child,
                            );

                            traverse_children = false;
                            cursor.goto_first_child();

                            if let Some(name) = node.child_by_field_name("name") {
                                Self::add_token(
                                    ctx,
                                    ctx.token_types.method,
                                    &name,
                                );

                                cursor.goto_next_sibling();
                            }

                            if cursor.goto_next_sibling() {
                                Self::collect_tokens_step(
                                    cursor,
                                    ctx,
                                );
                            }
                        } else {
                            if let Some(name) = node.child_by_field_name("name") {
                                Self::add_token(
                                    ctx,
                                    ctx.token_types.method,
                                    &name,
                                );
                            }
                        }
                    },
                    "string" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.string,
                            &node,
                        );
                    },
                    "pragma_strict" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.pragma_strict,
                            &node,
                        );
                    },
                    "param" => {
                        if let Some(child) = node.child_by_field_name("type") {
                            Self::add_token(
                                ctx,
                                ctx.token_types.parameter_type,
                                &child,
                            );
                        }

                        if let Some(child) = node.child_by_field_name("name") {
                            Self::add_token(
                                ctx,
                                ctx.token_types.parameter,
                                &child,
                            );
                        }
                    },
                    "appendto" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.appendto,
                            &node,
                        );
                    },
                    "include" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node,
                        );
                    },
                    "number" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.number,
                            &node,
                        );
                    },
                    "nil" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.nil,
                            &node,
                        );
                    },
                    "builtin_constant" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node,
                        );
                    },
                    "bool" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.bool,
                            &node,
                        );
                    },
                    "id" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.id,
                            &node,
                        );
                    },
                    "for_statement" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node.child(0).unwrap(),
                        );
                    },
                    "while_statement" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node.child(0).unwrap(),
                        );
                    },
                    "if_statement" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node.child(0).unwrap(),
                        );
                    },
                    "else" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node,
                        );
                    },
                    "return_statement" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node.child(0).unwrap(),
                        );
                    },
                    "flow_control_statement" => {
                        Self::add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node,
                        );
                    },
                    _ => {},
                }
            }

            if traverse_children && cursor.goto_first_child() {
                Self::collect_tokens_step(
                    cursor,
                    ctx,
                );
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }

}

#[cfg(test)]
mod tests {

    use tree_sitter::Parser;

    use super::*;

    #[test]
    fn should_have_token_for_comment() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("//comment", None).unwrap();

        let tokens = Highlighter::collect_tokens(&tree, TokenTypes::default());

        assert!(tokens.len() > 0);
    }

    #[test]
    fn should_not_crash_for_method_call_on_id() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("func GetX() { CLNK::Explode(); }", None).unwrap();

        let tokens = Highlighter::collect_tokens(&tree, TokenTypes::default());

        assert!(tokens.len() > 0);
    }

    #[test]
    fn should_not_crash_for_incomplete_method_call_on_id() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("/*-- Feurige Himmel --*/

        #strict
        
        func Initialize() 
        {
          SpreadDragons();
          CLNK::DoStuff();
        }
        ", None).unwrap();

        let tokens = Highlighter::collect_tokens(&tree, TokenTypes::default());

        assert!(tokens.len() > 0);
    }

    #[test]
    fn should_not_crash() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("func GetX() { Explode(100); }", None).unwrap();
        let tokens = Highlighter::collect_tokens(&tree, TokenTypes::default());

        assert!(tokens.len() > 0);
    }
}
