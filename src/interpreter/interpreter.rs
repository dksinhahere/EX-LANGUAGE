use crate::interpreter::error::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::parser::ast::{Expr, Literal, Stmt};
use crate::values::values::{ControlFlow, Environment, Function, Value};
use std::collections::HashMap;


#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
    // Map: visible_block_name -> HashMap<var_name, Value>
    pub(crate) visible: HashMap<String, HashMap<String, Value>>,
    // Track which visible blocks have been initialized
    pub(crate) initialized_visible: HashMap<String, bool>,
    // Store the initialization expressions for visible blocks
    pub(crate) visible_definitions: HashMap<String, Vec<(String, Expr)>>,
    // Track the current function context (to enforce visible block access)
    pub(crate) current_function_context: Option<Vec<String>>, // Current function's allowed visible blocks
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            visible: HashMap::new(),
            initialized_visible: HashMap::new(),
            visible_definitions: HashMap::new(),
            current_function_context: None,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    // fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
    //     match stmt {
    //         Stmt::Expression(expr) => {
    //             let _ = self.eval(expr)?;
    //             Ok(())
    //         }

    //         Stmt::SmartLock { variable } => {
    //             let value = self.environment.get(variable)?;
    //             self.environment.define_smart_lock(variable, value)?;
    //             Ok(())
    //         }

    //         Stmt::SmartUnlock { variable } => {
    //             let value = self.environment.get(variable)?;
    //             self.environment.define_smart_unclock(variable, value)?;
    //             Ok(())
    //         }

    //         Stmt::SmartKill { variable } => {
    //             self.environment.delete_variable(variable)?;
    //             Ok(())
    //         }

    //         Stmt::SmartRevive { variable } => {
    //             self.environment.define(variable, Value::Nil)?;
    //             Ok(())
    //         }

    //         Stmt::SmartConst { variable } => {
    //             let value = self.environment.get(variable)?;
    //             self.environment.define_constant(variable, value)?;
    //             Ok(())
    //         }

    //         Stmt::If {
    //             condition,
    //             then_branch,
    //             elif_branches,
    //             else_branch,
    //         } => {
    //             // Evaluate main if condition
    //             let condition_value = self.eval(condition)?;

    //             if condition_value.truthy() {
    //                 // Execute then branch
    //                 for stmt in then_branch {
    //                     self.execute(stmt)?;
    //                 }
    //             } else {
    //                 // Check elif branches
    //                 let mut executed = false;

    //                 for (elif_condition, elif_body) in elif_branches {
    //                     let elif_value = self.eval(elif_condition)?;

    //                     if elif_value.truthy() {
    //                         for stmt in elif_body {
    //                             self.execute(stmt)?;
    //                         }
    //                         executed = true;
    //                         break;
    //                     }
    //                 }

    //                 // Execute else branch if no elif was executed
    //                 #[allow(clippy::collapsible_if)]
    //                 if !executed {
    //                     if let Some(else_body) = else_branch {
    //                         for stmt in else_body {
    //                             self.execute(stmt)?;
    //                         }
    //                     }
    //                 }
    //             }

    //             Ok(())
    //         }

    //         Stmt::Label { _label_ } => {
    //             for label_item in _label_ {
    //                 let label_name = label_item.0.clone();
    //                 let is_callable = label_item.1;
    //                 let visible = label_item.2.clone();
    //                 let params = label_item.3.clone();
    //                 let args = label_item.4.clone();
    //                 let body = label_item.5.clone();

    //                 if is_callable {
    //                     // Store callable label as function in environment
    //                     let func = Value::Function(Function {
    //                         name: label_name.clone(),
    //                         params,
    //                         defaults: args,
    //                         body,
    //                         visible_blocks: visible,
    //                     });
    //                     self.environment.define(&label_name, func)?;
    //                 } else {
    //                     // Store control flow label in environment
    //                     let ctrl = Value::ControlFlow(ControlFlow {
    //                         name: label_name.clone(),
    //                         body,
    //                     });
    //                     self.environment.define(&label_name, ctrl)?;
    //                 }
    //             }
    //             Ok(())
    //         }
    //         Stmt::Jump { jump } => {
    //             // Get the target label from environment
    //             let target_value = self.environment.get(jump)?;

    //             match target_value {
    //                 Value::ControlFlow(ctrl) => {
    //                     // Execute the control flow label's body
    //                     self.environment.push_scope();
    //                     for stmt in &ctrl.body {
    //                         self.execute(stmt)?;
    //                     }
    //                     self.environment.pop_scope();
    //                     Ok(())
    //                 }
    //                 _ => Err(RuntimeError::custom(format!(
    //                     "'{}' is not a valid jump target (must be a control flow label)",
    //                     jump
    //                 ))),
    //             }
    //         }

    //         Stmt::Pass => {
    //             // Do nothing - pass statement
    //             Ok(())
    //         }

    //         Stmt::For {
    //             iterator,
    //             iterable,
    //             body,
    //         } => {
    //             // Evaluate iterable expression
    //             let iter_val = self.eval(iterable)?;

    //             match iter_val {
    //                 Value::Array(items) => {
    //                     // For-loop runs in its own scope (optional but clean)
    //                     self.environment.push_scope();

    //                     for item in items {
    //                         // Each iteration can get its own nested scope (optional).
    //                         // If you want iterator variable to be updated in same scope, remove this push/pop.
    //                         self.environment.push_scope();

    //                         // Bind iterator variable
    //                         self.environment.define(iterator, item)?;

    //                         // Execute body
    //                         for stmt in body {
    //                             self.execute(stmt)?;
    //                         }

    //                         self.environment.pop_scope();
    //                     }

    //                     self.environment.pop_scope();
    //                     Ok(())
    //                 }

    //                 _ => Err(RuntimeError::custom(format!(
    //                     "For-loop expects an Array iterable, got {}",
    //                     iter_val.type_name()
    //                 ))),
    //             }
    //         }

    //         Stmt::While { condition, body } => {
    //             // Keep looping while condition is truthy
    //             while self.eval(condition)?.truthy() {
    //                 self.environment.push_scope();

    //                 for stmt in body {
    //                     self.execute(stmt)?;
    //                 }

    //                 self.environment.pop_scope();
    //             }
    //             Ok(())
    //         }

    //         Stmt::DoWhile { body, condition } => {
    //             // Execute body at least once
    //             loop {
    //                 self.environment.push_scope();

    //                 for stmt in body {
    //                     self.execute(stmt)?;
    //                 }

    //                 self.environment.pop_scope();

    //                 // Check condition after executing body
    //                 if !self.eval(condition)?.truthy() {
    //                     break;
    //                 }
    //             }
    //             Ok(())
    //         }

    //         Stmt::Visible { _name_, _block_ } => {
    //             // Visible blocks are NOT executed in global scope
    //             // They are only DEFINED here and will be initialized
    //             // when a function with permission first accesses them

    //             // Store the definition for later initialization
    //             self.visible_definitions
    //                 .insert(_name_.clone(), _block_.clone());

    //             // Mark as defined but not initialized
    //             self.visible.insert(_name_.clone(), HashMap::new());
    //             self.initialized_visible.insert(_name_.clone(), false);

    //             Ok(())
    //         }

        
    //     }
    // }

    // fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
    //     match expr {
    //         Expr::_Literal_(lit) => Ok(self.literal_to_value(lit)),

    //         Expr::Grouping(inner) => self.eval(inner),

    //         Expr::MacroCall { var, body } => {
    //             for item in var.iter() {
    //                 self.eval(item)?;
    //             }
    //             for stmt in body.iter() {
    //                 self.execute(stmt)?;
    //             }

    //             Ok(Value::Bool(true))
    //         },

    //         Expr::DictionaryAccess { dict, member } => {
    //             // Get the dictionary value
    //             let dict_value = self.environment.get(dict)?;
                
    //             // Ensure it's actually a dictionary
    //             let dict_map = match dict_value {
    //                 Value::Dictionary(map) => map,
    //                 other => return Err(RuntimeError::custom(
    //                     format!("Expected Dictionary, got {}", other.type_name())
    //                 )),
    //             };

    //             // Evaluate member expression and convert to string key
    //             let member_key = match self.eval(member)? { // Use ? instead of unwrap
    //                 Value::String(s) => s,
    //                 Value::Int(i) => i.to_string(),
    //                 Value::Float(f) => f.to_string(),
    //                 Value::Bool(b) => b.to_string(),
    //                 Value::Char(c) => c.to_string(),
    //                 other => return Err(RuntimeError::custom(
    //                     format!("Dictionary keys must be primitive types, got {}", other.type_name())
    //                 )),
    //             };

    //             // Get the value from the dictionary
    //             let value = dict_map.get(&member_key)
    //                 .ok_or_else(|| RuntimeError::custom(
    //                     format!("Key '{}' not found in dictionary", member_key)
    //                 ))?;

    //             // If it's a function, execute it; otherwise return the value
    //             match value {
    //                 Value::Function(func) => {
    //                     // Execute the function with no arguments
    //                     self.environment.push_scope();
                        
    //                     // Execute all statements in the function body
    //                     for stmt in &func.body {
    //                         self.execute(stmt)?; // execute returns (), not Value
    //                     }
                        
    //                     self.environment.pop_scope();
                        
    //                     // For now, return Nil since functions don't have explicit return values
    //                     Ok(Value::Nil)
    //                 }
    //                 other => Ok(other.clone()),
    //             }
    //         },

    //         Expr::Dictionary(entries) => {
    //             let mut dict_map: HashMap<String, Value> = HashMap::new();

    //             for (key_expr, value_expr) in entries {
    //                 // Evaluate the key
    //                 let key_value = self.eval(key_expr)?;
                    
    //                 // Convert key to string
    //                 let key_str = match key_value {
    //                     Value::String(s) => s,
    //                     Value::Int(i) => i.to_string(),
    //                     Value::Float(f) => f.to_string(),
    //                     Value::Bool(b) => b.to_string(),
    //                     Value::Char(c) => c.to_string(),
    //                     _ => return Err(RuntimeError::custom(
    //                         format!("Dictionary keys must be primitive types, got {}", key_value.type_name())
    //                     )),
    //                 };

    //                 // Evaluate the value
    //                 let value = self.eval(value_expr)?;
                    
    //                 dict_map.insert(key_str, value);
    //             }

    //             Ok(Value::Dictionary(dict_map))
    //         }

    //         #[allow(clippy::useless_format)]
    //         Expr::Function { name, params, body } => {
    //             // Create an anonymous function value
    //             let key_str:String = match self.eval(name).unwrap() {
    //                 Value::String(s) => s,
    //                 Value::Int(i) => i.to_string(),
    //                 Value::Float(f) => f.to_string(),
    //                 Value::Bool(b) => b.to_string(),
    //                 Value::Char(c) => c.to_string(),
    //                 _ => return Err(RuntimeError::custom(
    //                         format!("Dictionary keys must be primitive types")
    //                 )),
    //             };

    //             Ok(Value::Function(Function {
    //                 name: key_str,
    //                 params: params.clone(),
    //                 defaults: params.clone(),
    //                 body: body.clone(),
    //                 visible_blocks: Vec::new(),
    //             }))
    //         }

    //         Expr::Iterable { value } => {
    //             let mut out = Vec::new();
    //             for e in value {
    //                 out.push(Value::Int(*e));
    //             }
    //             Ok(Value::Array(out))
    //         }

    //         Expr::Unary { operator, right } => {
    //             let value = self.eval(right)?;

    //             match operator.kind {
    //                 TokenKind::Minus => match value {
    //                     Value::Float(n) => Ok(Value::Float(-n)),
    //                     Value::Int(n) => Ok(Value::Int(-n)),
    //                     _ => Err(RuntimeError::new(RuntimeErrorKind::InvalidUnaryOperation {
    //                         operator: "-".to_string(),
    //                         operand_type: value.type_name().to_string(),
    //                     })),
    //                 },
    //                 TokenKind::Bang => Ok(Value::Bool(!value.truthy())),
    //                 _ => Err(RuntimeError::custom(format!(
    //                     "Unsupported unary operator: {:?}",
    //                     operator.kind
    //                 ))),
    //             }
    //         }

    //         Expr::Binary {
    //             left,
    //             operator,
    //             right,
    //         } => {
    //             let left_val = self.eval(left)?;
    //             let right_val = self.eval(right)?;

    //             match operator.kind {
    //                 TokenKind::Plus => Self::add(left_val, right_val),
    //                 TokenKind::Minus => Self::num_op(left_val, right_val, |a, b| a - b, "-"),
    //                 TokenKind::Star => Self::num_op(left_val, right_val, |a, b| a * b, "*"),
    //                 TokenKind::Slash => {
    //                     // Check for division by zero
    //                     let is_zero = match &right_val {
    //                         Value::Int(0) => true,
    //                         Value::Float(f) if *f == 0.0 => true,
    //                         _ => false,
    //                     };

    //                     if is_zero {
    //                         return Err(RuntimeError::division_by_zero());
    //                     }

    //                     Self::num_op(left_val, right_val, |a, b| a / b, "/")
    //                 }
    //                 TokenKind::EqualEqual => Ok(Value::Bool(left_val == right_val)),
    //                 TokenKind::BangEqual => Ok(Value::Bool(left_val != right_val)),
    //                 TokenKind::Greater => Self::cmp(left_val, right_val, |a, b| a > b, ">"),
    //                 TokenKind::GreaterEqual => Self::cmp(left_val, right_val, |a, b| a >= b, ">="),
    //                 TokenKind::Less => Self::cmp(left_val, right_val, |a, b| a < b, "<"),
    //                 TokenKind::LessEqual => Self::cmp(left_val, right_val, |a, b| a <= b, "<="),
    //                 TokenKind::And => {
    //                     if !left_val.truthy() {
    //                         Ok(left_val)
    //                     } else {
    //                         Ok(right_val)
    //                     }
    //                 }
    //                 TokenKind::Or => {
    //                     if left_val.truthy() {
    //                         Ok(left_val)
    //                     } else {
    //                         Ok(right_val)
    //                     }
    //                 }
    //                 _ => Err(RuntimeError::custom(format!(
    //                     "Unsupported binary operator: {:?}",
    //                     operator.kind
    //                 ))),
    //             }
    //         }

    //         Expr::AllocateVariable { name, val } => {
    //             let val = self.eval(val)?;
    //             self.environment.define(name, val)?;
    //             Ok(Value::Nil)
    //         }

    //         #[allow(clippy::collapsible_if)]
    //         Expr::Variable { name } => {
    //             // Check if variable exists in environment
    //             if self.environment.exists(name) {
    //                 return self.environment.get(name);
    //             }

    //             // Check if it's a visible block variable
    //             // Only allow access if we're in a function context with permission
    //             if let Some(allowed_blocks) = &self.current_function_context {
    //                 // We're inside a function - check if this variable is from an allowed visible block
    //                 for block_name in allowed_blocks {
    //                     if let Some(variables) = self.visible.get(block_name) {
    //                         if let Some(value) = variables.get(name) {
    //                             // Variable found in an allowed visible block
    //                             return Ok(value.clone());
    //                         }
    //                     }
    //                 }
    //             }

    //             // Variable not found or not accessible
    //             Err(RuntimeError::undefined_variable(name))
    //         }

    //         Expr::Print(expr) => {
    //             let value = self.eval(expr)?;
    //             match value {
    //                 Value::BigInt(bi) => println!("{}", bi),
    //                 Value::Bool(bo) => println!("{}", bo),
    //                 Value::Char(ch) => println!("{}", ch),
    //                 Value::String(st) => println!("{}", st),
    //                 Value::Int(it) => println!("{}", it),
    //                 Value::Float(fl) => println!("{}", fl),
    //                 Value::Nil => println!("Nil"),
    //                 _ => {
    //                     println!("Unable to Render On Display")
    //                 }
    //             }
    //             Ok(Value::Nil)
    //         }

            

    //         Expr::FunctionCall { function, args } => {
    //             // Get function from environment
    //             let func_value = self.environment.get(function)?;

    //             match func_value {
    //                 Value::Function(func) => {
    //                     // === INITIALIZE VISIBLE BLOCKS FOR THIS FUNCTION ===
    //                     // Check if this function has access to any visible blocks
    //                     for visible_block_name in &func.visible_blocks {
    //                         // Check if the visible block exists
    //                         if !self.visible.contains_key(visible_block_name) {
    //                             return Err(RuntimeError::custom(format!(
    //                                 "Function '{}' references undefined visible block '{}'",
    //                                 function, visible_block_name
    //                             )));
    //                         }

    //                         // Initialize the visible block on FIRST access
    //                         let is_initialized = self
    //                             .initialized_visible
    //                             .get(visible_block_name)
    //                             .copied()
    //                             .unwrap_or(false);

    //                         if !is_initialized {
    //                             // Clone the definition to avoid borrow checker issues
    //                             let block_def =
    //                                 self.visible_definitions.get(visible_block_name).cloned();

    //                             if let Some(block_def) = block_def {
    //                                 // Create a temporary scope to evaluate the initialization expressions
    //                                 self.environment.push_scope();

    //                                 let mut value_map: HashMap<String, Value> = HashMap::new();

    //                                 for (var_name, var_expr) in &block_def {
    //                                     let value = self.eval(var_expr)?;
    //                                     value_map.insert(var_name.clone(), value);
    //                                 }

    //                                 self.environment.pop_scope();

    //                                 // Store the initialized values
    //                                 self.visible.insert(visible_block_name.clone(), value_map);
    //                                 self.initialized_visible
    //                                     .insert(visible_block_name.clone(), true);
    //                             } else {
    //                                 return Err(RuntimeError::custom(format!(
    //                                     "Visible block '{}' is declared but has no definition",
    //                                     visible_block_name
    //                                 )));
    //                             }
    //                         }
    //                     }

    //                     // Set the current function context (for access control)
    //                     let previous_context = self.current_function_context.clone();
    //                     self.current_function_context = Some(func.visible_blocks.clone());

    //                     // Create new scope for function execution
    //                     self.environment.push_scope();

    //                     // Inject visible block variables into the function scope
    //                     for visible_block_name in &func.visible_blocks {
    //                         if let Some(variables) = self.visible.get(visible_block_name) {
    //                             for (var_name, value) in variables {
    //                                 // Define these variables in the function scope
    //                                 self.environment.define(var_name, value.clone())?;
    //                             }
    //                         }
    //                     }

    //                     // Build argument map from call-site arguments
    //                     let mut arg_map: HashMap<String, Value> = HashMap::new();
    //                     for (arg_name, arg_expr) in args {
    //                         let arg_value = self.eval(arg_expr)?;
    //                         arg_map.insert(arg_name.clone(), arg_value);
    //                     }

    //                     // Map external parameter names to internal variable names
    //                     for (i, external_param) in func.params.iter().enumerate() {
    //                         let internal_name = &func.defaults[i];

    //                         if let Some(arg_value) = arg_map.get(external_param) {
    //                             // Bind argument to internal variable name
    //                             self.environment.define(internal_name, arg_value.clone())?;
    //                         } else {
    //                             // Missing required parameter
    //                             self.environment.pop_scope();
    //                             self.current_function_context = previous_context;
    //                             return Err(RuntimeError::custom(format!(
    //                                 "Missing required parameter '{}' in function '{}'",
    //                                 external_param, function
    //                             )));
    //                         }
    //                     }

    //                     // Execute function body
    //                     for stmt in &func.body {
    //                         self.execute(stmt)?;
    //                     }

    //                     // IMPORTANT: Save back any modifications to visible block variables
    //                     // before popping the scope
    //                     for visible_block_name in &func.visible_blocks {
    //                         if let Some(variables) = self.visible.get_mut(visible_block_name) {
    //                             for (var_name, _) in variables.clone() {
    //                                 // Get the potentially modified value from the environment
    //                                 if let Ok(new_value) = self.environment.get(&var_name) {
    //                                     variables.insert(var_name, new_value);
    //                                 }
    //                             }
    //                         }
    //                     }

    //                     // Pop scope and restore previous context
    //                     self.environment.pop_scope();
    //                     self.current_function_context = previous_context;

    //                     Ok(Value::Nil)
    //                 }
    //                 _ => Err(RuntimeError::custom(format!(
    //                     "'{}' is not callable (type: {})",
    //                     function,
    //                     func_value.type_name()
    //                 ))),
    //             }
    //         }

    //         _ => Err(RuntimeError::custom("Unsupported expression")),
    //     }
    // }

    pub(crate) fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Int(i) => Value::Int(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::BigInt(s) => Value::BigInt(s.clone()),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Char(c) => Value::Char(*c),
            Literal::Nil => Value::Nil,
        }
    }

    pub(crate) fn add(left: Value, right: Value) -> RuntimeResult<Value> {
        match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::invalid_binary_op(
                "+",
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    pub(crate) fn num_op<F>(left: Value, right: Value, op: F, op_str: &str) -> RuntimeResult<Value>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (&left, &right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(op(*a, *b))),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Float(op(*a as f64, *b as f64))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(op(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(op(*a, *b as f64))),
            _ => Err(RuntimeError::invalid_binary_op(
                op_str,
                left.type_name(),
                right.type_name(),
            )),
        }
    }

    pub(crate) fn cmp<F>(left: Value, right: Value, op: F, op_str: &str) -> RuntimeResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (&left, &right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(op(*a, *b))),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(op(*a as f64, *b as f64))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(op(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(op(*a, *b as f64))),
            _ => Err(RuntimeError::invalid_binary_op(
                op_str,
                left.type_name(),
                right.type_name(),
            )),
        }
    }
}
