use log::debug;
use tower_lsp::lsp_types::SemanticToken;
use tree_sitter::{Tree, TreeCursor, Node};

use crate::TokenTypes;

#[derive(Debug, Default)]
struct Context {
    collection: Vec<SemanticToken>,
    last_line: u32,
    last_start: u32,
    token_types: TokenTypes,
}

pub fn collect_tokens(tree: &Tree, lut: TokenTypes) -> Vec<SemanticToken> {
    let mut cursor = tree.walk();
    let mut ctx = Context {
        token_types: lut,
        ..Context::default()
    };

    collect_tokens_step(&mut cursor, &mut ctx);
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

        if node.is_named() {
            match node.kind() {
                "comment" => {
                    add_token(
                        ctx,
                        ctx.token_types.comment,
                        &node,
                    );
                },

                "var_definition" => {
                    if let Some(ref child) = node.child(0) {
                        add_token(ctx, ctx.token_types.var_scope, child);
                    }
                },
                "function_definition" => {
                    let mut idx_of_func = 0;

                    if let Some(vis) = node.child_by_field_name("visibility") {
                        add_token(
                            ctx,
                            ctx.token_types.function,
                            &vis,
                        );

                        idx_of_func += 1;
                    }

                    if let Some(name) = node.child(idx_of_func) {
                        add_token(
                            ctx,
                            ctx.token_types.function,
                            &name,
                        );
                    }

                    if let Some(name) = node.child_by_field_name("name") {
                        add_token(
                            ctx,
                            ctx.token_types.method,
                            &name,
                        );  
                    }
                },
                "method_call" => {

                    if let Some(name) = node.child_by_field_name("name") {
                        add_token(
                            ctx,
                            ctx.token_types.method,
                            &name,
                        );  
                    }
                },
                "string" => {
                    add_token(
                        ctx,
                        ctx.token_types.string,
                        &node,
                    );
                },
                "pragma_strict" => {
                    add_token(
                        ctx,
                        ctx.token_types.pragma_strict,
                        &node,
                    );
                },
                "param" => {
                    // First child is type and optional
                    if node.child_count() > 1 {
                        add_token(
                            ctx,
                            ctx.token_types.keyword,
                            &node.child(0).unwrap(),
                        );

                        add_token(
                            ctx,
                            ctx.token_types.parameter,
                            &node.child(1).unwrap(),
                        );
                    } else {
                        add_token(
                            ctx,
                            ctx.token_types.parameter,
                            &node.child(0).unwrap(),
                        );
                    }
                },
                "appendto" => {
                    add_token(
                        ctx,
                        ctx.token_types.appendto,
                        &node,
                    );
                },
                "include" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node,
                    );
                },
                "number" => {
                    add_token(
                        ctx,
                        ctx.token_types.number,
                        &node,
                    );
                },
                "nil" => {
                    add_token(
                        ctx,
                        ctx.token_types.nil,
                        &node,
                    );
                },
                "builtin_constant" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node,
                    );
                },
                "bool" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node,
                    );
                },
                "id" => {
                    add_token(
                        ctx,
                        ctx.token_types.id,
                        &node,
                    );
                },
                "for_statement" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node.child(0).unwrap(),
                    );
                },
                "while_statement" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node.child(0).unwrap(),
                    );
                },
                "if_statement" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node.child(0).unwrap(),
                    );
                },
                "else" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node,
                    );
                },
                "return_statement" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node.child(0).unwrap(),
                    );
                },
                "flow_control_statement" => {
                    add_token(
                        ctx,
                        ctx.token_types.keyword,
                        &node,
                    );
                },
                _ => {},
            }
        }

        if cursor.goto_first_child() {
            collect_tokens_step(
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

#[cfg(test)]
mod tests {

    use std::{fs, io::Read};

    use tree_sitter::Parser;

    use crate::TokenTypes;

    use super::collect_tokens;

    #[test]
    fn should_have_token_for_comment() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("//comment", None).unwrap();
        // let tree = parser.parse("local name = \"Twonky\";", None).unwrap();

        let errors = collect_tokens(&tree, TokenTypes::default());

        assert!(errors.len() > 0);
    }

    #[test]
    fn should_have_error_for_nonsense() {

        let mut file = fs::File::open("/home/fmi/Desktop/test").unwrap();

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse(content, None).unwrap();
 
        let errors = collect_tokens(&tree, TokenTypes::default());

        assert!(errors.len() > 0);
    }
}
