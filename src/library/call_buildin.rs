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

            "array_new" => Some(Self::array_new()),
            "array_len" => Some(Self::array_len(args.get("src")?.clone())),
            "array_is_empty" => Some(Self::array_is_empty(args.get("src")?.clone())),

            "array_get" => Some(Self::array_get(
                args.get("src")?.clone(),
                args.get("idx")?.clone(),
            )),
            "array_set" => Some(Self::array_set(
                args.get("src")?.clone(),
                args.get("idx")?.clone(),
                args.get("value")?.clone(),
            )),

            "array_push" => Some(Self::array_push(
                args.get("src")?.clone(),
                args.get("value")?.clone(),
            )),
            "array_pop" => Some(Self::array_pop(args.get("src")?.clone())),

            "array_insert" => Some(Self::array_insert(
                args.get("src")?.clone(),
                args.get("idx")?.clone(),
                args.get("value")?.clone(),
            )),
            "array_remove" => Some(Self::array_remove(
                args.get("src")?.clone(),
                args.get("idx")?.clone(),
            )),

            "array_clear" => Some(Self::array_clear(args.get("src")?.clone())),
            "array_clone" => Some(Self::array_clone(args.get("src")?.clone())),

            "array_slice" => Some(Self::array_slice(
                args.get("src")?.clone(),
                args.get("start")?.clone(),
                args.get("end")?.clone(),
            )),

            "array_concat" => Some(Self::array_concat(
                args.get("a")?.clone(),
                args.get("b")?.clone(),
            )),

            "array_reverse" => Some(Self::array_reverse(args.get("src")?.clone())),
            "array_sort" => Some(Self::array_sort(args.get("src")?.clone())),

            "array_find" => Some(Self::array_find(
                args.get("src")?.clone(),
                args.get("value")?.clone(),
            )),

            "array_contains" => Some(Self::array_contains(
                args.get("src")?.clone(),
                args.get("value")?.clone(),
            )),


            _ => None,
        }
    }
}
