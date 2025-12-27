use crate::interpreter::interpreter::Interpreter;
use crate::parser::ast::Expr;
use crate::interpreter::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::values::values::Value;
use std::collections::HashMap;
use crate::values::values::Function;
use crate::lexer::TokenKind;

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
            },

            Expr::DictionaryAccess { dict, member } => {
                // Get the dictionary value
                let dict_value = self.environment.get(dict)?;
                
                // Ensure it's actually a dictionary
                let dict_map = match dict_value {
                    Value::Dictionary(map) => map,
                    other => return Err(RuntimeError::custom(
                        format!("Expected Dictionary, got {}", other.type_name())
                    )),
                };

                // Evaluate member expression and convert to string key
                let member_key = match self.eval(member)? { // Use ? instead of unwrap
                    Value::String(s) => s,
                    Value::Int(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Char(c) => c.to_string(),
                    other => return Err(RuntimeError::custom(
                        format!("Dictionary keys must be primitive types, got {}", other.type_name())
                    )),
                };

                // Get the value from the dictionary
                let value = dict_map.get(&member_key)
                    .ok_or_else(|| RuntimeError::custom(
                        format!("Key '{}' not found in dictionary", member_key)
                    ))?;

                // If it's a function, execute it; otherwise return the value
                match value {
                    Value::Function(func) => {
                        // Execute the function with no arguments
                        self.environment.push_scope();
                        
                        // Execute all statements in the function body
                        for stmt in &func.body {
                            self.execute(stmt)?; // execute returns (), not Value
                        }
                        
                        self.environment.pop_scope();
                        
                        // For now, return Nil since functions don't have explicit return values
                        Ok(Value::Nil)
                    }
                    other => Ok(other.clone()),
                }
            },

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
                        _ => return Err(RuntimeError::custom(
                            format!("Dictionary keys must be primitive types, got {}", key_value.type_name())
                        )),
                    };

                    // Evaluate the value
                    let value = self.eval(value_expr)?;
                    
                    dict_map.insert(key_str, value);
                }

                Ok(Value::Dictionary(dict_map))
            }

            #[allow(clippy::useless_format)]
            Expr::Function { name, params, body } => {
                // Create an anonymous function value
                let key_str:String = match self.eval(name).unwrap() {
                    Value::String(s) => s,
                    Value::Int(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Char(c) => c.to_string(),
                    _ => return Err(RuntimeError::custom(
                            format!("Dictionary keys must be primitive types")
                    )),
                };

                Ok(Value::Function(Function {
                    name: key_str,
                    params: params.clone(),
                    defaults: params.clone(),
                    body: body.clone(),
                    visible_blocks: Vec::new(),
                }))
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
                // Get function from environment
                let func_value = self.environment.get(function)?;

                match func_value {
                    Value::Function(func) => {
                        // === INITIALIZE VISIBLE BLOCKS FOR THIS FUNCTION ===
                        // Check if this function has access to any visible blocks
                        for visible_block_name in &func.visible_blocks {
                            // Check if the visible block exists
                            if !self.visible.contains_key(visible_block_name) {
                                return Err(RuntimeError::custom(format!(
                                    "Function '{}' references undefined visible block '{}'",
                                    function, visible_block_name
                                )));
                            }

                            // Initialize the visible block on FIRST access
                            let is_initialized = self
                                .initialized_visible
                                .get(visible_block_name)
                                .copied()
                                .unwrap_or(false);

                            if !is_initialized {
                                // Clone the definition to avoid borrow checker issues
                                let block_def =
                                    self.visible_definitions.get(visible_block_name).cloned();

                                if let Some(block_def) = block_def {
                                    // Create a temporary scope to evaluate the initialization expressions
                                    self.environment.push_scope();

                                    let mut value_map: HashMap<String, Value> = HashMap::new();

                                    for (var_name, var_expr) in &block_def {
                                        let value = self.eval(var_expr)?;
                                        value_map.insert(var_name.clone(), value);
                                    }

                                    self.environment.pop_scope();

                                    // Store the initialized values
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

                        // Create new scope for function execution
                        self.environment.push_scope();

                        // Inject visible block variables into the function scope
                        for visible_block_name in &func.visible_blocks {
                            if let Some(variables) = self.visible.get(visible_block_name) {
                                for (var_name, value) in variables {
                                    // Define these variables in the function scope
                                    self.environment.define(var_name, value.clone())?;
                                }
                            }
                        }

                        // Build argument map from call-site arguments
                        let mut arg_map: HashMap<String, Value> = HashMap::new();
                        for (arg_name, arg_expr) in args {
                            let arg_value = self.eval(arg_expr)?;
                            arg_map.insert(arg_name.clone(), arg_value);
                        }

                        // Map external parameter names to internal variable names
                        for (i, external_param) in func.params.iter().enumerate() {
                            let internal_name = &func.defaults[i];

                            if let Some(arg_value) = arg_map.get(external_param) {
                                // Bind argument to internal variable name
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

                        // IMPORTANT: Save back any modifications to visible block variables
                        // before popping the scope
                        for visible_block_name in &func.visible_blocks {
                            if let Some(variables) = self.visible.get_mut(visible_block_name) {
                                for (var_name, _) in variables.clone() {
                                    // Get the potentially modified value from the environment
                                    if let Ok(new_value) = self.environment.get(&var_name) {
                                        variables.insert(var_name, new_value);
                                    }
                                }
                            }
                        }

                        // Pop scope and restore previous context
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