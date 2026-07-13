//! Environment chain for lexical scopes (Crafting Interpreters Ch.8 / book Ch.6).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

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
