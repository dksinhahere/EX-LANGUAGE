use crate::interpreter::error::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::parser::ast::Stmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub defaults: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ControlFlow {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i128),
    Float(f64),
    BigInt(String),
    String(String),
    Bool(bool),
    Char(char),
    Nil,
    Function(Function),
    ControlFlow(ControlFlow),
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq for ControlFlow {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(n) => *n != 0.0 && !n.is_nan(),
            Value::BigInt(s) => s != "0" && !s.is_empty(),
            Value::String(s) => !s.is_empty(),
            Value::Char(_) => true,
            Value::Function(_) => true,
            Value::ControlFlow(_) => true,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::BigInt(_) => "BigInt",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::Char(_) => "Char",
            Value::Nil => "Nil",
            Value::Function(_) => "Function",
            Value::ControlFlow(_) => "ControlFlow",
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    is_constant: bool,
    smart_lock: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn define(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                if binding.is_constant {
                    return Err(RuntimeError::cannot_reassign_constant(name));
                }
                if binding.smart_lock {
                    return Err(RuntimeError::cannot_reassign_smart_locked(name));
                }
                binding.value = value;
                return Ok(());
            }
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: false,
                    smart_lock: false,
                },
            );
        }

        Ok(())
    }

    pub fn define_constant(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: true,
                    smart_lock: false,
                },
            );
        }

        Ok(())
    }

    pub fn define_smart_lock(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: false,
                    smart_lock: true,
                },
            );
        }

        Ok(())
    }

    pub fn define_smart_unclock(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.remove(name);

            scope.insert(
                name.to_string(),
                Binding {
                    value,
                    is_constant: false,
                    smart_lock: false,
                },
            );
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> RuntimeResult<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Ok(binding.value.clone());
            }
        }
        Err(RuntimeError::undefined_variable(name))
    }

    pub fn delete_variable(&mut self, name: &str) -> RuntimeResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get(name) {
                if binding.is_constant {
                    return Err(RuntimeError::cannot_delete_constant(name));
                }
                if binding.smart_lock {
                    return Err(RuntimeError::cannot_delete_smart_locked(name));
                }

                scope.remove(name);
                return Ok(());
            }
        }

        Err(RuntimeError::new(RuntimeErrorKind::CannotDeleteUndefined(
            name.to_string(),
        )))
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}