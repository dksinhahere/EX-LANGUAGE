use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::lexer::TokenKind;
use crate::parser::ast::Expr;
use crate::values::values::Value;
use std::collections::HashMap;

impl Interpreter {
    pub(crate) fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::_Literal_(lit) => Ok(self.literal_to_value(lit)),
            Expr::Grouping(inner) => self.eval(inner),
            Expr::MacroCall { var, body } => {
                for item in var.iter() {
                    self.eval(item)?;
                }
                for stmt in body.iter() {
                    self.execute(stmt)?;
                }

                Ok(Value::Bool(true))
            }


            Expr::Array { elements } => {
                let mut arr = Vec::new();
                for element in elements {
                    let value = self.eval(element)?;
                    arr.push(value);
                }
                Ok(Value::Array(arr))
            }

            Expr::Axis { elements } => {
                let mut axis = Vec::new();
                for element in elements {
                    let value = self.eval(element)?;
                    axis.push(value);
                }
                Ok(Value::Axis(axis))
            }

            #[allow(clippy::needless_return)]
            Expr::Access { ds, member } => {
                // Start from the root value
                let mut current = self.environment.get(ds)?;

                // Helper: convert Value -> dictionary key string
                let to_dict_key = |v: Value| -> RuntimeResult<String> {
                    Ok(match v {
                        Value::Int(i) => i.to_string(),
                        Value::String(s) => s,
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Char(ch) => ch.to_string(),
                        Value::BigInt(bi) => bi, // assuming BigInt is stored as String in your Value
                        _ => {
                            return Err(RuntimeError::custom(format!(
                                "Invalid key type for dictionary access: {}",
                                v.type_name()
                            )));
                        }
                    })
                };

                // Helper: resolve negative indexing
                let resolve_index = |idx: i128, len: usize, what: &str| -> RuntimeResult<usize> {
                    if len == 0 {
                        return Err(RuntimeError::custom(format!("Index {} out of bounds for {} of length 0", idx, what)));
                    }

                    if idx < 0 {
                        let l = len as i128;
                        if idx.abs() > l {
                            return Err(RuntimeError::custom(format!(
                                "Index {} out of bounds for {} of length {}",
                                idx, what, len
                            )));
                        }
                        Ok((l + idx) as usize)
                    } else {
                        let u = idx as usize;
                        if u >= len {
                            return Err(RuntimeError::custom(format!(
                                "Index {} out of bounds for {} of length {}",
                                idx, what, len
                            )));
                        }
                        Ok(u)
                    }
                };

                for item in member.iter() {
                    let accessor = self.eval(item)?;

                    current = match current {
                        Value::Dictionary(dict) => {
                            let key = to_dict_key(accessor)?;
                            dict.get(&key).cloned().ok_or_else(|| {
                                RuntimeError::custom(format!("Key '{}' not found in dictionary", key))
                            })?
                        }

                        Value::Array(arr) => {
                            let idx = match accessor {
                                Value::Int(i) => i,
                                _ => {
                                    return Err(RuntimeError::custom(format!(
                                        "Array index must be integer, got {}",
                                        accessor.type_name()
                                    )));
                                }
                            };

                            let actual = resolve_index(idx, arr.len(), "array")?;
                            arr.get(actual).cloned().ok_or_else(|| {
                                RuntimeError::custom(format!(
                                    "Index {} out of bounds for array of length {}",
                                    idx, arr.len()
                                ))
                            })?
                        }

                        Value::Axis(axis) => {
                            let idx = match accessor {
                                Value::Int(i) => i,
                                _ => {
                                    return Err(RuntimeError::custom(format!(
                                        "Axis index must be integer, got {}",
                                        accessor.type_name()
                                    )));
                                }
                            };

                            let actual = resolve_index(idx, axis.len(), "axis")?;
                            axis.get(actual).cloned().ok_or_else(|| {
                                RuntimeError::custom(format!(
                                    "Index {} out of bounds for axis of length {}",
                                    idx, axis.len()
                                ))
                            })?
                        }

                        other => {
                            return Err(RuntimeError::custom(format!(
                                "Cannot access member on type '{}'",
                                other.type_name()
                            )));
                        }
                    };
                }

                Ok(current)
            }


            Expr::Dictionary(entries) => {
                let mut dict_map: HashMap<String, Value> = HashMap::new();

                for (key_expr, value_expr) in entries {
                    // Evaluate the key
                    let key_value = self.eval(key_expr)?;

                    // Convert key to string
                    let key_str = match key_value {
                        Value::String(s) => s,
                        Value::Int(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Char(c) => c.to_string(),
                        _ => {
                            return Err(RuntimeError::custom(format!(
                                "Dictionary keys must be primitive types, got {}",
                                key_value.type_name()
                            )));
                        }
                    };

                    // Evaluate the value
                    let value = self.eval(value_expr)?;

                    dict_map.insert(key_str, value);
                }

                Ok(Value::Dictionary(dict_map))
            }

            Expr::Iterable { value } => {
                let mut out = Vec::new();
                for e in value {
                    out.push(Value::Int(*e));
                }
                Ok(Value::Array(out))
            }

            Expr::Unary { operator, right } => {
                let value = self.eval(right)?;

                match operator.kind {
                    TokenKind::Minus => match value {
                        Value::Float(n) => Ok(Value::Float(-n)),
                        Value::Int(n) => Ok(Value::Int(-n)),
                        _ => Err(RuntimeError::new(RuntimeErrorKind::InvalidUnaryOperation {
                            operator: "-".to_string(),
                            operand_type: value.type_name().to_string(),
                        })),
                    },
                    TokenKind::Bang => Ok(Value::Bool(!value.truthy())),
                    _ => Err(RuntimeError::custom(format!(
                        "Unsupported unary operator: {:?}",
                        operator.kind
                    ))),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;

                match operator.kind {
                    TokenKind::Plus => Self::add(left_val, right_val),
                    TokenKind::Minus => Self::num_op(left_val, right_val, |a, b| a - b, "-"),
                    TokenKind::Star => Self::num_op(left_val, right_val, |a, b| a * b, "*"),
                    TokenKind::Slash => {
                        // Check for division by zero
                        let is_zero = match &right_val {
                            Value::Int(0) => true,
                            Value::Float(f) if *f == 0.0 => true,
                            _ => false,
                        };

                        if is_zero {
                            return Err(RuntimeError::division_by_zero());
                        }

                        Self::num_op(left_val, right_val, |a, b| a / b, "/")
                    }
                    TokenKind::EqualEqual => Ok(Value::Bool(left_val == right_val)),
                    TokenKind::BangEqual => Ok(Value::Bool(left_val != right_val)),
                    TokenKind::Greater => Self::cmp(left_val, right_val, |a, b| a > b, ">"),
                    TokenKind::GreaterEqual => Self::cmp(left_val, right_val, |a, b| a >= b, ">="),
                    TokenKind::Less => Self::cmp(left_val, right_val, |a, b| a < b, "<"),
                    TokenKind::LessEqual => Self::cmp(left_val, right_val, |a, b| a <= b, "<="),
                    TokenKind::And => {
                        if !left_val.truthy() {
                            Ok(left_val)
                        } else {
                            Ok(right_val)
                        }
                    }
                    TokenKind::Or => {
                        if left_val.truthy() {
                            Ok(left_val)
                        } else {
                            Ok(right_val)
                        }
                    }
                    _ => Err(RuntimeError::custom(format!(
                        "Unsupported binary operator: {:?}",
                        operator.kind
                    ))),
                }
            }

            Expr::AllocateVariable { name, val } => {
                let val = self.eval(val)?;
                self.environment.define(name, val)?;
                Ok(Value::Nil)
            }

            #[allow(clippy::collapsible_if)]
            Expr::Variable { name } => {
                // Check if variable exists in environment
                if self.environment.exists(name) {
                    return self.environment.get(name);
                }

                // Check if it's a visible block variable
                // Only allow access if we're in a function context with permission
                if let Some(allowed_blocks) = &self.current_function_context {
                    // We're inside a function - check if this variable is from an allowed visible block
                    for block_name in allowed_blocks {
                        if let Some(variables) = self.visible.get(block_name) {
                            if let Some(value) = variables.get(name) {
                                // Variable found in an allowed visible block
                                return Ok(value.clone());
                            }
                        }
                    }
                }

                // Variable not found or not accessible
                Err(RuntimeError::undefined_variable(name))
            }

            Expr::Print(expr) => {
                let value = self.eval(expr)?;
                match value {
                    Value::BigInt(bi) => println!("{}", bi),
                    Value::Bool(bo) => println!("{}", bo),
                    Value::Char(ch) => println!("{}", ch),
                    Value::String(st) => println!("{}", st),
                    Value::Int(it) => println!("{}", it),
                    Value::Float(fl) => println!("{}", fl),
                    Value::Nil => println!("Nil"),
                    _ => {
                        println!("Unable to Render On Display")
                    }
                }
                Ok(Value::Nil)
            }

            Expr::FunctionCall { function, args } => {
                // --------------------------------------------
                // 1) Evaluate call-site arguments FIRST
                // --------------------------------------------
                let mut evaluated_args: HashMap<String, Value> = HashMap::new();
                for (arg_name, arg_expr) in args {
                    let arg_value = self.eval(arg_expr)?;
                    evaluated_args.insert(arg_name.clone(), arg_value);
                }

                // --------------------------------------------
                // 2) Builtin/standard function check FIRST
                // --------------------------------------------
                if let Some(result) = self.call_builtin(function, &evaluated_args) {
                    return result;
                }

                // --------------------------------------------
                // 3) Fallback: User-defined function
                // --------------------------------------------
                let func_value = self.environment.get(function)?;

                match func_value {
                    Value::Function(func) => {
                        // === INITIALIZE VISIBLE BLOCKS FOR THIS FUNCTION ===
                        for visible_block_name in &func.visible_blocks {
                            if !self.visible.contains_key(visible_block_name) {
                                return Err(RuntimeError::custom(format!(
                                    "Label '{}' references undefined visible block '{}'",
                                    function, visible_block_name
                                )));
                            }

                            let is_initialized = self
                                .initialized_visible
                                .get(visible_block_name)
                                .copied()
                                .unwrap_or(false);

                            if !is_initialized {
                                let block_def = self.visible_definitions.get(visible_block_name).cloned();

                                if let Some(block_def) = block_def {
                                    // temp scope for init expressions
                                    self.environment.push_scope();

                                    let mut value_map: HashMap<String, Value> = HashMap::new();
                                    for (var_name, var_expr) in &block_def {
                                        let value = self.eval(var_expr)?;
                                        value_map.insert(var_name.clone(), value);
                                    }

                                    self.environment.pop_scope();

                                    self.visible.insert(visible_block_name.clone(), value_map);
                                    self.initialized_visible
                                        .insert(visible_block_name.clone(), true);
                                } else {
                                    return Err(RuntimeError::custom(format!(
                                        "Visible block '{}' is declared but has no definition",
                                        visible_block_name
                                    )));
                                }
                            }
                        }

                        // Set the current function context (for access control)
                        let previous_context = self.current_function_context.clone();
                        self.current_function_context = Some(func.visible_blocks.clone());

                        // New scope for function execution
                        self.environment.push_scope();

                        // Inject visible block variables into the function scope
                        for visible_block_name in &func.visible_blocks {
                            if let Some(variables) = self.visible.get(visible_block_name) {
                                for (var_name, value) in variables {
                                    self.environment.define(var_name, value.clone())?;
                                }
                            }
                        }

                        // Map call-site args (evaluated_args) to internal parameter names
                        // func.params: external param names
                        // func.defaults: internal variable names (as you described)
                        for (i, external_param) in func.params.iter().enumerate() {
                            let internal_name = &func.defaults[i];

                            if let Some(arg_value) = evaluated_args.get(external_param) {
                                self.environment.define(internal_name, arg_value.clone())?;
                            } else {
                                // Missing required parameter
                                self.environment.pop_scope();
                                self.current_function_context = previous_context;
                                return Err(RuntimeError::custom(format!(
                                    "Missing required parameter '{}' in function '{}'",
                                    external_param, function
                                )));
                            }
                        }

                        // Execute function body
                        for stmt in &func.body {
                            self.execute(stmt)?;
                        }

                        // Save back modifications to visible block vars
                        for visible_block_name in &func.visible_blocks {
                            if let Some(variables) = self.visible.get_mut(visible_block_name) {
                                // clone keys to avoid borrow issues
                                let keys: Vec<String> = variables.keys().cloned().collect();
                                for var_name in keys {
                                    if let Ok(new_value) = self.environment.get(&var_name) {
                                        variables.insert(var_name, new_value);
                                    }
                                }
                            }
                        }

                        // Pop scope + restore context
                        self.environment.pop_scope();
                        self.current_function_context = previous_context;

                        Ok(Value::Nil)
                    }

                    _ => Err(RuntimeError::custom(format!(
                        "'{}' is not callable (type: {})",
                        function,
                        func_value.type_name()
                    ))),
                }
            }


            _ => Err(RuntimeError::custom("Unsupported expression")),
        }
    }
}
