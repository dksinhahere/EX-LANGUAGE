use std::collections::HashMap;

use super::Value;

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    constant: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    // scopes[0] = global, last = current
    scopes: Vec<HashMap<String, Binding>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return; // keep global
        }
        self.scopes.pop();
    }

    /// Define variable in CURRENT scope
    pub fn define(&mut self, name: impl Into<String>, value: Value, constant: bool) {
        let scope = self
            .scopes
            .last_mut()
            .expect("Environment must always have at least one scope");

        scope.insert(
            name.into(),
            Binding {
                value,
                constant,
            },
        );
    }

    /// Get variable (searches from inner -> global)
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Some(b.value.clone());
            }
        }
        None
    }

    /// Assign existing variable; returns false if not found OR if constant
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(b) = scope.get_mut(name) {
                if b.constant {
                    return false;
                }
                b.value = value;
                return true;
            }
        }
        false
    }

    pub fn current_keys(&self) -> Vec<String> {
        self.scopes
            .last()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}
