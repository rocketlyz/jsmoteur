//! Tree-walk interpreter (expressions Ch.7 + statements/state Ch.8).
//!
//! Semantics (documented for this subset):
//! - `+`: Number+Number add; String+String concat; else runtime error (no coercion).
//! - `==` / `!=` / `===` / `!==`: same strict equality (tag + content).
//! - Truthy (`!`): only `null` and `false` are falsy; `0` and `""` are truthy
//!   (ponytail: JS-simplified; upgrade to full JS falsy later if needed).
//! - Uninitialized `var`/`let`/`const` → `null`.
//! - `console.log` → `Stmt::Print` (host stdout / collected output).
//! - `if` / `while`; `for` desugared in parser; `&&` / `||` short-circuit (JS-style values).

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Literal, Stmt};
use crate::env::Environment;
use crate::token::TokenKind;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    /// Captured `console.log` lines (also printed to stdout when `echo` is true).
    pub output: Vec<String>,
    echo: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
            output: Vec::new(),
            echo: true,
        }
    }

    pub fn new_silent() -> Self {
        Interpreter {
            environment: Environment::new(),
            output: Vec::new(),
            echo: false,
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                let line = value.to_string();
                if self.echo {
                    println!("{}", line);
                }
                self.output.push(line);
                Ok(())
            }
            Stmt::Var {
                kind: _,
                name,
                initializer,
            } => {
                let value = match initializer {
                    Some(init) => self.evaluate(init)?,
                    None => Value::Null,
                };
                self.environment
                    .borrow_mut()
                    .define(name.clone(), value);
                Ok(())
            }
            Stmt::Block(stmts) => {
                let previous = Rc::clone(&self.environment);
                self.environment = Environment::child(Rc::clone(&previous));
                let result = (|| {
                    for s in stmts {
                        self.execute(s)?;
                    }
                    Ok(())
                })();
                self.environment = previous;
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(then_branch)
                } else if let Some(els) = else_branch {
                    self.execute(els)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition)?) {
                    self.execute(body)?;
                }
                Ok(())
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(literal_to_value(lit)),
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Variable(name) => self
                .environment
                .borrow()
                .get(name)
                .map_err(|message| RuntimeError { message }),
            Expr::Assign { name, value } => {
                let v = self.evaluate(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name, v.clone())
                    .map_err(|message| RuntimeError { message })?;
                Ok(v)
            }
            Expr::Unary { op, right } => {
                let r = self.evaluate(right)?;
                match op {
                    TokenKind::Sub => match r {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(err("Operand must be a number for unary '-'.")),
                    },
                    TokenKind::Not => Ok(Value::Bool(!is_truthy(&r))),
                    _ => Err(err(&format!("Unknown unary operator {:?}", op))),
                }
            }
            Expr::Logical { left, op, right } => {
                let l = self.evaluate(left)?;
                match op {
                    TokenKind::Or => {
                        if is_truthy(&l) {
                            Ok(l)
                        } else {
                            self.evaluate(right)
                        }
                    }
                    TokenKind::And => {
                        if !is_truthy(&l) {
                            Ok(l)
                        } else {
                            self.evaluate(right)
                        }
                    }
                    _ => Err(err(&format!("Unknown logical operator {:?}", op))),
                }
            }
            Expr::Binary { left, op, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                binary(op, l, r)
            }
        }
    }
}

/// Convenience: run statements with echoing prints (for `main`).
pub fn interpret(stmts: &[Stmt]) -> Result<Vec<String>, RuntimeError> {
    let mut interp = Interpreter::new();
    interp.interpret(stmts)?;
    Ok(interp.output)
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
    use crate::value::Value;

    fn run(source: &str) -> Result<Interpreter, RuntimeError> {
        let tokens = Scanner::new(source).scan_tokens();
        let stmts = Parser::new(tokens)
            .parse()
            .expect("parse should succeed in these tests");
        let mut interp = Interpreter::new_silent();
        interp.interpret(&stmts)?;
        Ok(interp)
    }

    #[test]
    fn evals_precedence() {
        let interp = run("1 + 2 * 3;").unwrap();
        // expression stmt: no print; just ensure no error
        assert!(interp.output.is_empty());
    }

    #[test]
    fn evals_string_concat_via_print() {
        let interp = run(r#"console.log("a" + "b");"#).unwrap();
        assert_eq!(interp.output, vec!["ab".to_string()]);
    }

    #[test]
    fn evals_unary_negate() {
        let interp = run("console.log(-(4));").unwrap();
        assert_eq!(interp.output, vec!["-4".to_string()]);
    }

    #[test]
    fn rejects_mixed_add() {
        let err = match run(r#""a" + 1;"#) {
            Err(e) => e,
            Ok(_) => panic!("expected runtime error"),
        };
        assert!(err.message.contains("numbers or two strings"));
    }

    #[test]
    fn ch8_acceptance_var_and_log() {
        let interp = run("var a = 1;\nvar b = a + 3;\nconsole.log(b);").unwrap();
        assert_eq!(interp.output, vec!["4".to_string()]);
    }

    #[test]
    fn block_scope_shadowing() {
        let interp = run(
            "var a = 1;\n{ var a = 2; console.log(a); }\nconsole.log(a);",
        )
        .unwrap();
        assert_eq!(interp.output, vec!["2".to_string(), "1".to_string()]);
    }

    #[test]
    fn assignment_updates() {
        let interp = run("var a = 1;\na = a + 1;\nconsole.log(a);").unwrap();
        assert_eq!(interp.output, vec!["2".to_string()]);
    }

    #[test]
    fn can_read_defined_value() {
        let interp = run("var x = 7; console.log(x);").unwrap();
        assert_eq!(
            interp.environment.borrow().get("x").unwrap(),
            Value::Number(7.0)
        );
    }

    #[test]
    fn ch9_acceptance_while_if() {
        let interp = run(
            "var i = 0;\nwhile (i < 3) { i = i + 1; }\nif (i === 3) { console.log(i); }",
        )
        .unwrap();
        assert_eq!(interp.output, vec!["3".to_string()]);
    }

    #[test]
    fn logical_and_short_circuits() {
        let interp = run("var x = 0;\nfalse && (x = 1);\nconsole.log(x);").unwrap();
        assert_eq!(interp.output, vec!["0".to_string()]);
    }

    #[test]
    fn logical_or_short_circuits() {
        let interp = run("var x = 0;\ntrue || (x = 1);\nconsole.log(x);").unwrap();
        assert_eq!(interp.output, vec!["0".to_string()]);
    }

    #[test]
    fn for_desugar_prints() {
        let interp =
            run("for (var i = 0; i < 3; i = i + 1) { console.log(i); }").unwrap();
        assert_eq!(
            interp.output,
            vec!["0".to_string(), "1".to_string(), "2".to_string()]
        );
    }
}
