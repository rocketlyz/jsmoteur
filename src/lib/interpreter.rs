//! Tree-walk interpreter (Ch.7–10).
//!
//! Semantics (documented for this subset):
//! - `+`: Number+Number add; String+String concat; else runtime error (no coercion).
//! - `==` / `!=` / `===` / `!==`: same strict equality (tag + content).
//! - Truthy (`!`): only `null` and `false` are falsy; `0` and `""` are truthy.
//! - Uninitialized `var`/`let`/`const` → `null`.
//! - `console.log` → `Stmt::Print`.
//! - `if` / `while`; `for` desugared in parser; `&&` / `||` short-circuit.
//! - Functions: `Value::Function` + Call + Return (arity must match).

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Literal, Stmt};
use crate::env::{Environment, Value};
use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

enum ExecResult {
    Continue,
    Return(Value),
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
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
            match self.execute(stmt)? {
                ExecResult::Continue => {}
                ExecResult::Return(_) => {
                    return Err(err("Cannot return from top-level code."));
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<ExecResult, RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(ExecResult::Continue)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                let line = value.to_string();
                if self.echo {
                    println!("{}", line);
                }
                self.output.push(line);
                Ok(ExecResult::Continue)
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
                Ok(ExecResult::Continue)
            }
            Stmt::Block(stmts) => self.execute_block(stmts, Environment::child(Rc::clone(&self.environment))),
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
                    Ok(ExecResult::Continue)
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition)?) {
                    match self.execute(body)? {
                        ExecResult::Continue => {}
                        ExecResult::Return(v) => return Ok(ExecResult::Return(v)),
                    }
                }
                Ok(ExecResult::Continue)
            }
            Stmt::Function { name, params, body } => {
                let function = Value::Function {
                    name: name.clone(),
                    params: Rc::from(params.as_slice()),
                    body: Rc::from(body.as_slice()),
                    closure: Rc::clone(&self.environment),
                };
                self.environment
                    .borrow_mut()
                    .define(name.clone(), function);
                Ok(ExecResult::Continue)
            }
            Stmt::Return { value } => {
                let v = match value {
                    Some(e) => self.evaluate(e)?,
                    None => Value::Null,
                };
                Ok(ExecResult::Return(v))
            }
        }
    }

    fn execute_block(
        &mut self,
        stmts: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<ExecResult, RuntimeError> {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;
        let mut result = Ok(ExecResult::Continue);
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(ExecResult::Continue) => {}
                Ok(ExecResult::Return(v)) => {
                    result = Ok(ExecResult::Return(v));
                    break;
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }
        self.environment = previous;
        result
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
            Expr::Call { callee, arguments } => {
                let callee_val = self.evaluate(callee)?;
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
                self.call_value(callee_val, args)
            }
        }
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match callee {
            Value::Function {
                name,
                params,
                body,
                closure,
            } => {
                if args.len() != params.len() {
                    return Err(err(&format!(
                        "Expected {} arguments but got {} for '{}'.",
                        params.len(),
                        args.len(),
                        name
                    )));
                }
                let frame = Environment::child(closure);
                {
                    let mut env = frame.borrow_mut();
                    for (param, arg) in params.iter().zip(args.into_iter()) {
                        env.define(param.clone(), arg);
                    }
                }
                match self.execute_block(&body, frame)? {
                    ExecResult::Return(v) => Ok(v),
                    ExecResult::Continue => Ok(Value::Null),
                }
            }
            _ => Err(err("Can only call functions.")),
        }
    }
}

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
    a == b
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

    #[test]
    fn ch10_acceptance_add() {
        let interp = run(
            "function add(a, b) { return a + b; }\nconsole.log(add(1, 2));",
        )
        .unwrap();
        assert_eq!(interp.output, vec!["3".to_string()]);
    }

    #[test]
    fn arity_mismatch_errors() {
        let err = match run("function f(a) { return a; }\nf();") {
            Err(e) => e,
            Ok(_) => panic!("expected arity error"),
        };
        assert!(err.message.contains("Expected 1 arguments"));
    }

    #[test]
    fn test_js_style_acc() {
        let interp = run(
            "var a = 1;\nvar b = a + 3;\nfunction acc(a, b) { console.log(a + b); }\nacc(a, b);",
        )
        .unwrap();
        assert_eq!(interp.output, vec!["5".to_string()]);
    }
}
