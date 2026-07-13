//! Expression AST for the tree-walk interpreter.
//!
//! Traversal style (project-wide): `enum` + `match` — no Visitor/`accept`.
//! Stmt nodes land in Ch.8; Variable/Call/Assign when the parser needs them.

use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: TokenKind,
        right: Box<Expr>,
    },
    Unary {
        op: TokenKind,
        right: Box<Expr>,
    },
    Literal(Literal),
    Grouping(Box<Expr>),
}

/// Lisp-style tree dump for debugging (book's AstPrinter analogue).
pub fn pretty_print(expr: &Expr) -> String {
    match expr {
        Expr::Binary { left, op, right } => {
            format!(
                "({} {} {})",
                op_lexeme(op),
                pretty_print(left),
                pretty_print(right)
            )
        }
        Expr::Unary { op, right } => {
            format!("({} {})", op_lexeme(op), pretty_print(right))
        }
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => {
                // Integers print without trailing .0 for readable trees.
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Literal::String(s) => format!("{:?}", s),
            Literal::Bool(b) => b.to_string(),
            Literal::Null => "null".to_string(),
        },
        Expr::Grouping(inner) => format!("(group {})", pretty_print(inner)),
    }
}

fn op_lexeme(op: &TokenKind) -> &'static str {
    match op {
        TokenKind::Add => "+",
        TokenKind::Sub => "-",
        TokenKind::Mul => "*",
        TokenKind::Div => "/",
        TokenKind::Mod => "%",
        TokenKind::Not => "!",
        TokenKind::And => "&&",
        TokenKind::Or => "||",
        TokenKind::LT => "<",
        TokenKind::GT => ">",
        TokenKind::LE => "<=",
        TokenKind::GE => ">=",
        TokenKind::Eq => "==",
        TokenKind::NotEq => "!=",
        TokenKind::EqStrict => "===",
        TokenKind::NotEqStrict => "!==",
        TokenKind::Assign => "=",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_prints_binary_add() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1.0))),
            op: TokenKind::Add,
            right: Box::new(Expr::Literal(Literal::Number(2.0))),
        };
        assert_eq!(pretty_print(&expr), "(+ 1 2)");
    }

    #[test]
    fn pretty_prints_unary_and_grouping() {
        // -(4) → (- (group 4))
        let expr = Expr::Unary {
            op: TokenKind::Sub,
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                4.0,
            ))))),
        };
        assert_eq!(pretty_print(&expr), "(- (group 4))");
    }

    #[test]
    fn pretty_prints_nested_precedence_shape() {
        // 1 + 2 * 3  as tree with * under +
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1.0))),
            op: TokenKind::Add,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(2.0))),
                op: TokenKind::Mul,
                right: Box::new(Expr::Literal(Literal::Number(3.0))),
            }),
        };
        assert_eq!(pretty_print(&expr), "(+ 1 (* 2 3))");
    }
}
