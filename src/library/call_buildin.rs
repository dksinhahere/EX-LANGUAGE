use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use std::collections::HashMap;


impl Interpreter {
    /// Return Some(result) if builtin exists, else None to fall back to user functions.
    pub(crate) fn call_builtin(&mut self,name: &str,args: &HashMap<String, Value>) -> Option<RuntimeResult<Value>> {
        match name {
            
            "print" => {
                
                for v in args.values() {
                    match v {
                        Value::Int(i) => print!("{}", i),
                        Value::UInt(u) => print!("{}", u),
                        Value::Float(f) => print!("{}", f),
                        Value::BigInt(s) => print!("{}", s),
                        Value::String(s) => print!("{}", s),
                        Value::Bool(b) => print!("{}", b),
                        Value::Char(c) => print!("{}", c),
                        Value::Nil => print!("nil"),

                        Value::Function(_) => print!("<function>"),
                        Value::ControlFlow(_) => print!("<control-flow>"),

                        Value::Array(arr) => {
                            print!("[");
                            for (i, val) in arr.iter().enumerate() {
                                if i > 0 {
                                    print!(", ");
                                }
                                print!("{:?}", val);
                            }
                            print!("]");
                        }

                        Value::Dictionary(map) => {
                            print!("{{");
                            let mut first = true;
                            for (k, v) in map {
                                if !first {
                                    print!(", ");
                                }
                                first = false;
                                print!("{}: {:?}", k, v);
                            }
                            print!("}}");
                        }

                        Value::Axis(arr) => {
                            print!("axis(");
                            for (i, val) in arr.iter().enumerate() {
                                if i > 0 {
                                    print!(", ");
                                }
                                print!("{:?}", val);
                            }
                            print!(")");
                        }
                    }
                }
                println!();
                Some(Ok(Value::Nil))
                
            }

            "typeof" => {
                if args.len() != 1 {
                    return Some(Err(RuntimeError::custom(
                        "typeof expects exactly 1 argument",
                    )));
                }
                let v = args.get("src")?;
                Some(Ok(Value::String(v.type_name().to_string())))
            }

            "cast_type" => {
                let value = match args.get("value") {
                    Some(v) => v.clone(),
                    None => {
                        return Some(Err(RuntimeError::custom(
                            "cast_type missing argument 'value'",
                        )))
                    }
                };

                let target_type = match args.get("type") {
                    Some(v) => v.clone(),
                    None => {
                        return Some(Err(RuntimeError::custom(
                            "cast_type missing argument 'type'",
                        )))
                    }
                };

                Some(Self::cast_type(value, target_type))
            }

            _ => None,
        }
    }
}
