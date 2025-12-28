
use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::{RuntimeError, RuntimeResult};

impl Interpreter {
    pub(crate) fn cast_type(value: Value, target_type: Value) -> RuntimeResult<Value> {
        
        let t:String = match target_type {
            Value::String(s) => s.to_string(),
            _=> Err(RuntimeError::custom("Expected target_type as string"))?
        };

        match t.as_str() {
            // -----------------------------
            // INT
            // -----------------------------
            "INT" | "INTEGER" => match value {
                Value::Int(i) => Ok(Value::Int(i)),
                Value::UInt(u) => {
                    if u <= i128::MAX as u128 {
                        Ok(Value::Int(u as i128))
                    } else {
                        Err(RuntimeError::custom(
                            "Cannot cast UInt to Int: overflow",
                        ))?
                    }
                }
                Value::Float(f) => Ok(Value::Int(f as i128)),
                Value::Bool(b) => Ok(Value::Int(if b { 1 } else { 0 })),
                Value::Char(c) => Ok(Value::Int(c as u32 as i128)),
                Value::String(s) => s.parse::<i128>().map(Value::Int).map_err(|_| {
                    RuntimeError::custom(
                        format!("Cannot cast string '{}' to Int", s),
                    )
                }),
                Value::Nil => Ok(Value::Int(0)),
                other => Err(RuntimeError::custom(
                    format!("Cannot cast {} to Int", other.type_name()),
                )),
            },

            // -----------------------------
            // UINT
            // -----------------------------
            "UINT" | "UINTEGER" => match value {
                Value::UInt(u) => Ok(Value::UInt(u)),
                Value::Int(i) => {
                    if i >= 0 {
                        Ok(Value::UInt(i as u128))
                    } else {
                        Err(RuntimeError::custom
(
                            "Cannot cast negative Int to UInt",
                        ))
                    }
                }
                Value::Float(f) => {
                    if f.is_sign_negative() {
                        Err(RuntimeError::custom
(
                            "Cannot cast negative Float to UInt",
                        ))
                    } else {
                        Ok(Value::UInt(f as u128))
                    }
                }
                Value::Bool(b) => Ok(Value::UInt(if b { 1 } else { 0 })),
                Value::Char(c) => Ok(Value::UInt(c as u32 as u128)),
                Value::String(s) => s.parse::<u128>().map(Value::UInt).map_err(|_| {
                    RuntimeError::custom
(
                        format!("Cannot cast string '{}' to UInt", s),
                    )
                }),
                Value::Nil => Ok(Value::UInt(0)),
                other => Err(RuntimeError::custom
(
                    format!("Cannot cast {} to UInt", other.type_name()),
                )),
            },

            // -----------------------------
            // FLOAT
            // -----------------------------
            "FLOAT" => match value {
                Value::Float(f) => Ok(Value::Float(f)),
                Value::Int(i) => Ok(Value::Float(i as f64)),
                Value::UInt(u) => Ok(Value::Float(u as f64)),
                Value::Bool(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
                Value::Char(c) => Ok(Value::Float(c as u32 as f64)),
                Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                    RuntimeError::custom
(
                        format!("Cannot cast string '{}' to Float", s),
                    )
                }),
                Value::Nil => Ok(Value::Float(0.0)),
                other => Err(RuntimeError::custom
(
                    format!("Cannot cast {} to Float", other.type_name()),
                )),
            },

            // -----------------------------
            // BOOL
            // -----------------------------
            "BOOL" | "BOOLEAN" => match value {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Nil => Ok(Value::Bool(false)),
                Value::Int(i) => Ok(Value::Bool(i != 0)),
                Value::UInt(u) => Ok(Value::Bool(u != 0)),
                Value::Float(f) => Ok(Value::Bool(f != 0.0)),
                Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                Value::Char(c) => Ok(Value::Bool(c != '\0')),
                other => Err(RuntimeError::custom
(
                    
                    format!("Cannot cast {} to Bool", other.type_name()),
                )),
            },

            // -----------------------------
            // STRING
            // -----------------------------
            "STR" | "STRING" => match value {
                Value::String(s) => Ok(Value::String(s)),
                Value::Int(i) => Ok(Value::String(i.to_string())),
                Value::UInt(u) => Ok(Value::String(u.to_string())),
                Value::Float(f) => Ok(Value::String(f.to_string())),
                Value::Bool(b) => Ok(Value::String(b.to_string())),
                Value::Char(c) => Ok(Value::String(c.to_string())),
                Value::Nil => Ok(Value::String("nil".into())),
                other => Err(RuntimeError::custom
(
                    format!("Cannot cast {} to String", other.type_name()),
                )),
            },

            // -----------------------------
            // CHAR
            // -----------------------------
            "CHAR" | "CHARACTER" => match value {
                Value::Char(c) => Ok(Value::Char(c)),
                Value::Int(i) => {
                    let u = i as u32;
                    char::from_u32(u).map(Value::Char).ok_or_else(|| {
                        RuntimeError::custom
(
                            "Invalid codepoint for Char",
                        )
                    })
                }
                Value::UInt(u) => {
                    if u <= u32::MAX as u128 {
                        char::from_u32(u as u32).map(Value::Char).ok_or_else(|| {
                            RuntimeError::custom
(
                                "Invalid codepoint for Char",
                            )
                        })
                    } else {
                        Err(RuntimeError::custom
(
                            "Invalid UInt for Char",
                        ))
                    }
                }
                Value::String(s) => {
                    let mut it = s.chars();
                    match (it.next(), it.next()) {
                        (Some(c), None) => Ok(Value::Char(c)),
                        _ => Err(RuntimeError::custom
(
                            "String must contain exactly 1 character to cast to Char",
                        )),
                    }
                }
                other => Err(RuntimeError::custom
(
                    format!("Cannot cast {} to Char", other.type_name()),
                )),
            },

            // -----------------------------
            // NIL
            // -----------------------------
            "NIL" | "NULL" => Ok(Value::Nil),

            #[allow(clippy::useless_format)]
            _ => Err(RuntimeError::custom(
                format!("Unknown target type").as_str(),
            ))?,
        }
    }
}