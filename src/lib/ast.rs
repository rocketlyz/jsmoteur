//! Expression / statement AST for the tree-walk interpreter.
//!
//! Traversal style (project-wide): `enum` + `match` — no Visitor/`accept`.

use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: TokenKind,
        right: Box<Expr>,
    },
    Logical {
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
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        kind: VarKind,
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
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
        Expr::Logical { left, op, right } => {
            let name = match op {
                TokenKind::And => "and",
                TokenKind::Or => "or",
                _ => "?",
            };
            format!("({} {} {})", name, pretty_print(left), pretty_print(right))
        }
        Expr::Unary { op, right } => {
            format!("({} {})", op_lexeme(op), pretty_print(right))
        }
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => {
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
        Expr::Variable(name) => name.clone(),
        Expr::Assign { name, value } => format!("(assign {} {})", name, pretty_print(value)),
        Expr::Call { callee, arguments } => {
            let args: Vec<_> = arguments.iter().map(pretty_print).collect();
            format!("(call {} {})", pretty_print(callee), args.join(" "))
        }
    }
}

pub fn pretty_print_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Expression(e) => pretty_print(e),
        Stmt::Print(e) => format!("(print {})", pretty_print(e)),
        Stmt::Var {
            kind,
            name,
            initializer,
        } => {
            let k = match kind {
                VarKind::Var => "var",
                VarKind::Let => "let",
                VarKind::Const => "const",
            };
            match initializer {
                Some(init) => format!("({} {} {})", k, name, pretty_print(init)),
                None => format!("({} {})", k, name),
            }
        }
        Stmt::Block(stmts) => {
            let body: Vec<_> = stmts.iter().map(pretty_print_stmt).collect();
            format!("(block {})", body.join(" "))
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => match else_branch {
            Some(els) => format!(
                "(if {} {} {})",
                pretty_print(condition),
                pretty_print_stmt(then_branch),
                pretty_print_stmt(els)
            ),
            None => format!(
                "(if {} {})",
                pretty_print(condition),
                pretty_print_stmt(then_branch)
            ),
        },
        Stmt::While { condition, body } => format!(
            "(while {} {})",
            pretty_print(condition),
            pretty_print_stmt(body)
        ),
        Stmt::Function { name, params, body } => {
            let params = params.join(" ");
            let body: Vec<_> = body.iter().map(pretty_print_stmt).collect();
            format!("(fun {} ({}) {})", name, params, body.join(" "))
        }
        Stmt::Return { value } => match value {
            Some(e) => format!("(return {})", pretty_print(e)),
            None => "(return)".to_string(),
        },
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
