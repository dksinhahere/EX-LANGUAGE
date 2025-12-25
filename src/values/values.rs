use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Char(char),
}

impl Value {
    /// Returns true if the value is "truthy" (follows JS-like rules)
    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Char(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    is_constant: bool,
}

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Define a new mutable variable in the current scope
    pub fn define(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: false,
                },
            );
        }
    }

    /// Define a new constant variable in the current scope
    pub fn define_constant(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: true,
                },
            );
        }
    }

    /// Get a variable's value (searches from innermost to outermost scope)
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(binding.value.clone());
            }
        }
        None
    }

    
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                if binding.is_constant {
                    return false; // Cannot reassign constant
                }
                binding.value = value;
                return true;
            }
        }
        false
    }

    /// Push a new scope (for blocks, functions, etc.)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}