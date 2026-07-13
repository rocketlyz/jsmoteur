//! Environment chain + runtime values (Crafting Interpreters Ch.8 / Ch.10).
//!
//! Value and Environment live together so `Value::Function` can hold a closure
//! env without a value ↔ env import cycle.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::Stmt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Function {
        name: String,
        params: Rc<[String]>,
        body: Rc<[Stmt]>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (
                Value::Function {
                    name: n1,
                    body: b1,
                    ..
                },
                Value::Function {
                    name: n2,
                    body: b2,
                    ..
                },
            ) => n1 == n2 && Rc::ptr_eq(b1, b2),
            _ => false,
        }
    }
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
            Value::Function { name, .. } => write!(f, "<fn {}>", name),
        }
    }
}

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn child(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value, String> {
        if let Some(v) = self.values.get(name) {
            return Ok(v.clone());
        }
        if let Some(ref outer) = self.enclosing {
            return outer.borrow().get(name);
        }
        Err(format!("Undefined variable '{}'.", name))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(ref outer) = self.enclosing {
            return outer.borrow_mut().assign(name, value);
        }
        Err(format!("Undefined variable '{}'.", name))
    }
}
