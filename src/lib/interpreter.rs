//! Tree-walk expression evaluator (Crafting Interpreters Ch.7 / book Ch.5).
//!
//! Semantics (documented for this subset):
//! - `+`: Number+Number add; String+String concat; else runtime error (no coercion).
//! - `==` / `!=` / `===` / `!==`: same strict equality (tag + content).
//! - Truthy (`!`): only `null` and `false` are falsy; `0` and `""` are truthy
//!   (ponytail: JS-simplified; upgrade to full JS falsy later if needed).

use std::fmt;

use crate::ast::{Expr, Literal};
use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

pub fn interpret(exprs: &[Expr]) -> Result<Vec<Value>, RuntimeError> {
    let mut out = Vec::with_capacity(exprs.len());
    for expr in exprs {
        out.push(evaluate(expr)?);
    }
    Ok(out)
}

fn evaluate(expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Literal(lit) => Ok(literal_to_value(lit)),
        Expr::Grouping(inner) => evaluate(inner),
        Expr::Unary { op, right } => {
            let r = evaluate(right)?;
            match op {
                TokenKind::Sub => match r {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(err("Operand must be a number for unary '-'.")),
                },
                TokenKind::Not => Ok(Value::Bool(!is_truthy(&r))),
                _ => Err(err(&format!("Unknown unary operator {:?}", op))),
            }
        }
        Expr::Binary { left, op, right } => {
            let l = evaluate(left)?;
            let r = evaluate(right)?;
            binary(op, l, r)
        }
    }
}

fn literal_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Number(n) => Value::Number(*n),
        Literal::String(s) => Value::String(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Null => Value::Null,
    }
}

fn binary(op: &TokenKind, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match op {
        TokenKind::Add => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(err("Operands must be two numbers or two strings for '+'.")),
        },
        TokenKind::Sub => nums(left, right, |a, b| a - b),
        TokenKind::Mul => nums(left, right, |a, b| a * b),
        TokenKind::Div => nums(left, right, |a, b| a / b),
        TokenKind::Mod => nums(left, right, |a, b| a % b),
        TokenKind::LT => cmp(left, right, |a, b| a < b),
        TokenKind::LE => cmp(left, right, |a, b| a <= b),
        TokenKind::GT => cmp(left, right, |a, b| a > b),
        TokenKind::GE => cmp(left, right, |a, b| a >= b),
        TokenKind::Eq | TokenKind::EqStrict => Ok(Value::Bool(is_equal(&left, &right))),
        TokenKind::NotEq | TokenKind::NotEqStrict => Ok(Value::Bool(!is_equal(&left, &right))),
        _ => Err(err(&format!("Unknown binary operator {:?}", op))),
    }
}

fn nums(left: Value, right: Value, f: impl Fn(f64, f64) -> f64) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        _ => Err(err("Operands must be numbers.")),
    }
}

fn cmp(left: Value, right: Value, f: impl Fn(f64, f64) -> bool) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
        _ => Err(err("Operands must be numbers.")),
    }
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        _ => true,
    }
}

fn is_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        _ => false,
    }
}

fn err(message: &str) -> RuntimeError {
    RuntimeError {
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Scanner;
    use crate::parser::Parser;

    fn eval_src(source: &str) -> Result<Vec<Value>, RuntimeError> {
        let tokens = Scanner::new(source).scan_tokens();
        let exprs = Parser::new(tokens)
            .parse()
            .expect("parse should succeed in these tests");
        interpret(&exprs)
    }

    #[test]
    fn evals_precedence() {
        let vals = eval_src("1 + 2 * 3;").unwrap();
        assert_eq!(vals, vec![Value::Number(7.0)]);
        assert_eq!(vals[0].to_string(), "7");
    }

    #[test]
    fn evals_string_concat() {
        let vals = eval_src(r#""a" + "b";"#).unwrap();
        assert_eq!(vals, vec![Value::String("ab".into())]);
        assert_eq!(vals[0].to_string(), "ab");
    }

    #[test]
    fn evals_unary_negate() {
        let vals = eval_src("-(4);").unwrap();
        assert_eq!(vals, vec![Value::Number(-4.0)]);
        assert_eq!(vals[0].to_string(), "-4");
    }

    #[test]
    fn rejects_mixed_add() {
        let err = eval_src(r#""a" + 1;"#).unwrap_err();
        assert!(err.message.contains("numbers or two strings"));
    }

    #[test]
    fn evals_ch6_file_style() {
        let vals = eval_src("1 + 2 * 3;\n-(4);").unwrap();
        assert_eq!(vals[0], Value::Number(7.0));
        assert_eq!(vals[1], Value::Number(-4.0));
    }
}
