use tree_sitter::{Point,  Tree, TreeCursor};

pub struct SyntaxError {
    start: Point,
    end: Point,
    message: String,
}

pub fn collect_errors(tree: &Tree) -> Vec<SyntaxError> {
    let mut errs = vec![];

    let mut cursor = tree.walk();

    collect_errors_step(&mut cursor, &mut errs);

    errs
}

fn collect_errors_step(
    cursor: &mut TreeCursor,
    errors: &mut Vec<SyntaxError>,
) {
    loop {
        let node = cursor.node();

        if node.is_error() {
            errors.push(SyntaxError {
                start: node.start_position(),                
                end: node.end_position(), 
                message: String::from("Syntax error"),
            });

            break;
        } else if node.has_error() {
            if cursor.goto_first_child() {
                collect_errors_step(cursor, errors);
            } else {
                errors.push(SyntaxError {
                    start: node.start_position(),                
                    end: node.end_position(), 
                    message: String::from("Syntax error somewhere here"),
                });
            }
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }

    cursor.goto_parent();
}

#[cfg(test)]
mod tests {

    use tree_sitter::Parser;

    use super::collect_errors;

    #[test]
    fn should_have_error_for_nonsense() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c4script::language()).expect("Loading c4scrpt grammar");

        let tree = parser.parse("sdfsdfd", None).unwrap();
 
        let errors = collect_errors(&tree);

        assert!(errors.len() > 0);
    }
}
