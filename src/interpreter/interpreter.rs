use crate::interpreter::error::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::lexer::TokenKind;
use crate::parser::ast::{Expr, Literal, Stmt};
use crate::values::values::{ControlFlow, Environment, Function, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
    // Map: visible_block_name -> HashMap<var_name, Value>
    visible: HashMap<String, HashMap<String, Value>>,
    // Track which visible blocks have been initialized
    initialized_visible: HashMap<String, bool>,
    // Store the initialization expressions for visible blocks
    visible_definitions: HashMap<String, Vec<(String, Expr)>>,
    // Track the current function context (to enforce visible block access)
    current_function_context: Option<Vec<String>>, // Current function's allowed visible blocks
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

    fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                let _ = self.eval(expr)?;
                Ok(())
            }

            Stmt::SmartLock { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_smart_lock(variable, value)?;
                Ok(())
            }

            Stmt::SmartUnlock { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_smart_unclock(variable, value)?;
                Ok(())
            }

            Stmt::SmartKill { variable } => {
                self.environment.delete_variable(variable)?;
                Ok(())
            }

            Stmt::SmartRevive { variable } => {
                self.environment.define(variable, Value::Nil)?;
                Ok(())
            }

            Stmt::SmartConst { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_constant(variable, value)?;
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            } => {
                // Evaluate main if condition
                let condition_value = self.eval(condition)?;

                if condition_value.truthy() {
                    // Execute then branch
                    for stmt in then_branch {
                        self.execute(stmt)?;
                    }
                } else {
                    // Check elif branches
                    let mut executed = false;

                    for (elif_condition, elif_body) in elif_branches {
                        let elif_value = self.eval(elif_condition)?;

                        if elif_value.truthy() {
                            for stmt in elif_body {
                                self.execute(stmt)?;
                            }
                            executed = true;
                            break;
                        }
                    }

                    // Execute else branch if no elif was executed
                    #[allow(clippy::collapsible_if)]
                    if !executed {
                        if let Some(else_body) = else_branch {
                            for stmt in else_body {
                                self.execute(stmt)?;
                            }
                        }
                    }
                }

                Ok(())
            }

            Stmt::Label { _label_ } => {
                for label_item in _label_ {
                    let label_name = label_item.0.clone();
                    let is_callable = label_item.1;
                    let visible = label_item.2.clone();
                    let params = label_item.3.clone();
                    let args = label_item.4.clone();
                    let body = label_item.5.clone();

                    if is_callable {
                        // Store callable label as function in environment
                        let func = Value::Function(Function {
                            name: label_name.clone(),
                            params,
                            defaults: args,
                            body,
                            visible_blocks: visible,
                        });
                        self.environment.define(&label_name, func)?;
                    } else {
                        // Store control flow label in environment
                        let ctrl = Value::ControlFlow(ControlFlow {
                            name: label_name.clone(),
                            body,
                        });
                        self.environment.define(&label_name, ctrl)?;
                    }
                }
                Ok(())
            }
            Stmt::Jump { jump } => {
                // Get the target label from environment
                let target_value = self.environment.get(jump)?;

                match target_value {
                    Value::ControlFlow(ctrl) => {
                        // Execute the control flow label's body
                        self.environment.push_scope();
                        for stmt in &ctrl.body {
                            self.execute(stmt)?;
                        }
                        self.environment.pop_scope();
                        Ok(())
                    }
                    _ => Err(RuntimeError::custom(format!(
                        "'{}' is not a valid jump target (must be a control flow label)",
                        jump
                    ))),
                }
            }

            Stmt::Pass => {
                // Do nothing - pass statement
                Ok(())
            }

            Stmt::For {
                iterator,
                iterable,
                body,
            } => {
                // Evaluate iterable expression
                let iter_val = self.eval(iterable)?;

                match iter_val {
                    Value::Array(items) => {
                        // For-loop runs in its own scope (optional but clean)
                        self.environment.push_scope();

                        for item in items {
                            // Each iteration can get its own nested scope (optional).
                            // If you want iterator variable to be updated in same scope, remove this push/pop.
                            self.environment.push_scope();

                            // Bind iterator variable
                            self.environment.define(iterator, item)?;

                            // Execute body
                            for stmt in body {
                                self.execute(stmt)?;
                            }

                            self.environment.pop_scope();
                        }

                        self.environment.pop_scope();
                        Ok(())
                    }

                    _ => Err(RuntimeError::custom(format!(
                        "For-loop expects an Array iterable, got {}",
                        iter_val.type_name()
                    ))),
                }
            }

            Stmt::While { condition, body } => {
                // Keep looping while condition is truthy
                while self.eval(condition)?.truthy() {
                    self.environment.push_scope();

                    for stmt in body {
                        self.execute(stmt)?;
                    }

                    self.environment.pop_scope();
                }
                Ok(())
            }

            Stmt::DoWhile { body, condition } => {
                // Execute body at least once
                loop {
                    self.environment.push_scope();

                    for stmt in body {
                        self.execute(stmt)?;
                    }

                    self.environment.pop_scope();

                    // Check condition after executing body
                    if !self.eval(condition)?.truthy() {
                        break;
                    }
                }
                Ok(())
            }

            Stmt::Visible { _name_, _block_ } => {
                // Visible blocks are NOT executed in global scope
                // They are only DEFINED here and will be initialized
                // when a function with permission first accesses them

                // Store the definition for later initialization
                self.visible_definitions
                    .insert(_name_.clone(), _block_.clone());

                // Mark as defined but not initialized
                self.visible.insert(_name_.clone(), HashMap::new());
                self.initialized_visible.insert(_name_.clone(), false);

                Ok(())
            }

            Stmt::StructDef { name, methods } => {
                let struct_def = Value::StructDef(crate::values::values::StructDef {
                    name: name.clone(),
                    methods: methods.clone(),
                });
                self.environment.define(name, struct_def)?;
                Ok(())
            }
        }
    }

    fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
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

            Expr::StructInstantiation {
                struct_name,
                method_name,
                args,
            } => {
                // Get the struct definition
                let struct_value = self.environment.get(struct_name)?;

                match struct_value {
                    Value::StructDef(struct_def) => {
                        // Create a new instance
                        let mut instance = crate::values::values::StructInstance {
                            struct_name: struct_name.clone(),
                            fields: std::collections::HashMap::new(),
                            methods: struct_def.methods.clone(),
                        };

                        // -----------------------------
                        // FIX: map ::new(...) -> constructor(...)
                        // -----------------------------
                        let lookup_name: &str = if method_name == "new" {
                            "constructor"
                        } else {
                            method_name.as_str()
                        };

                        // Find constructor (or method) and ERROR if missing
                        let constructor = struct_def
                            .methods
                            .iter()
                            .find(|m| m.name == lookup_name)
                            .ok_or_else(|| {
                                RuntimeError::custom(format!(
                                    "Runtime Error: Struct '{}' has no method '{}' (lookup '{}')",
                                    struct_name, method_name, lookup_name
                                ))
                            })?;

                        // Create a new scope for constructor execution
                        self.environment.push_scope();

                        // Bind 'self' to allow field initialization
                        let self_value = Value::StructInstance(instance.clone());
                        self.environment.define("self", self_value)?;

                        // Bind constructor parameters (skip 'self' which is first param)
                        let param_start = if !constructor.params.is_empty() && constructor.params[0] == "self" {
                            1
                        } else {
                            0
                        };

                        for (i, param) in constructor.params[param_start..].iter().enumerate() {
                            if i < args.len() {
                                let arg_value = self.eval(&args[i])?;
                                self.environment.define(param, arg_value)?;
                            } else {
                                // Optional strictness: missing arg -> runtime error
                                // return Err(RuntimeError::custom(format!(
                                //     "Runtime Error: Missing argument for parameter '{}' in '{}::{}'",
                                //     param, struct_name, method_name
                                // )));
                            }
                        }

                        // Execute constructor body
                        for stmt in &constructor.body {
                            self.execute(stmt)?;
                        }

                        // Extract fields that were set via self.field = value
                        match self.environment.get("self") {
                            Ok(Value::StructInstance(updated_instance)) => {
                                instance = updated_instance;
                            }
                            Ok(_) => {
                                self.environment.pop_scope();
                                return Err(RuntimeError::custom(
                                    "Runtime Error: 'self' was overwritten with a non-struct value",
                                ));
                            }
                            Err(e) => {
                                self.environment.pop_scope();
                                return Err(e);
                            }
                        }

                        self.environment.pop_scope();

                        Ok(Value::StructInstance(instance))
                    }

                    _ => Err(RuntimeError::custom(format!(
                        "'{}' is not a struct definition",
                        struct_name
                    ))),
                }
            }


            Expr::MemberAccess { object, member } => {
                let obj_value = self.eval(object)?;
                
                match obj_value {
                    Value::StructInstance(instance) => {
                        if let Some(field_value) = instance.fields.get(member) {
                            Ok(field_value.clone())
                        } else {
                            Err(RuntimeError::custom(format!(
                                "Struct '{}' has no field '{}'",
                                instance.struct_name, member
                            )))
                        }
                    }
                    _ => Err(RuntimeError::custom(format!(
                        "Cannot access member '{}' on non-struct type {}",
                        member,
                        obj_value.type_name()
                    ))),
                }
            }

            #[allow(clippy::collapsible_if)]
            Expr::MemberAssign {
                object,
                member,
                value,
            } => {
                // Special handling for self.field = value in methods
                if let Expr::Variable { name } = &**object {
                    if name == "self" {
                        // Get current self instance
                        if let Ok(Value::StructInstance(mut instance)) = self.environment.get("self") {
                            let new_value = self.eval(value)?;
                            instance.fields.insert(member.clone(), new_value.clone());
                            
                            // Update self in environment
                            self.environment.define("self", Value::StructInstance(instance))?;
                            
                            return Ok(Value::Nil);
                        }
                    }
                }
                
                // For regular object.field = value, we need to handle it differently
                // We need to get the variable name and update it
                if let Expr::Variable { name: var_name } = &**object {
                    let obj_value = self.environment.get(var_name)?;
                    
                    match obj_value {
                        Value::StructInstance(mut instance) => {
                            let new_value = self.eval(value)?;
                            instance.fields.insert(member.clone(), new_value);
                            
                            // Update the variable with the modified instance
                            self.environment.define(var_name, Value::StructInstance(instance))?;
                            
                            Ok(Value::Nil)
                        }
                        _ => Err(RuntimeError::custom(format!(
                            "Cannot assign to member '{}' on non-struct type {}",
                            member,
                            obj_value.type_name()
                        ))),
                    }
                } else {
                    Err(RuntimeError::custom(
                        "Member assignment requires a simple variable reference".to_string()
                    ))
                }
            }
        
            // NEW: Method call: obj.method(args)
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_value = self.eval(object)?;
                
                match obj_value {
                    Value::StructInstance(instance) => {
                        // Find the method
                        if let Some(method_def) = instance.methods.iter().find(|m| m.name == *method) {
                            // Create new scope for method execution
                            self.environment.push_scope();
                            
                            // Bind 'self' to the instance
                            self.environment.define("self", Value::StructInstance(instance.clone()))?;
                            
                            // Inject instance fields into scope
                            for (field_name, field_value) in &instance.fields {
                                self.environment.define(field_name, field_value.clone())?;
                            }
                            
                            // Bind method parameters (skip 'self' if it's first)
                            let param_start = if !method_def.params.is_empty() && method_def.params[0] == "self" {
                                1
                            } else {
                                0
                            };
                            
                            for (i, param) in method_def.params[param_start..].iter().enumerate() {
                                if i < args.len() {
                                    let arg_value = self.eval(&args[i])?;
                                    self.environment.define(param, arg_value)?;
                                }
                            }
                            
                            // Execute method body
                            for stmt in &method_def.body {
                                self.execute(stmt)?;
                            }
                            
                            // Extract updated fields - check self first
                            let mut updated_instance = instance.clone();
                            if let Ok(Value::StructInstance(self_instance)) = self.environment.get("self") {
                                updated_instance = self_instance;
                            } else {
                                // Fallback: extract fields from environment
                                for field_name in instance.fields.keys() {
                                    if let Ok(updated_value) = self.environment.get(field_name) {
                                        updated_instance.fields.insert(field_name.clone(), updated_value);
                                    }
                                }
                            }
                            
                            self.environment.pop_scope();
                            
                            // Update the original variable if this was called on a variable
                            if let Expr::Variable { name: var_name } = &**object {
                                self.environment.define(var_name, Value::StructInstance(updated_instance))?;
                            }
                            
                            Ok(Value::Nil)
                        } else {
                            Err(RuntimeError::custom(format!(
                                "Struct '{}' has no method '{}'",
                                instance.struct_name, method
                            )))
                        }
                    }
                    _ => Err(RuntimeError::custom(format!(
                        "Cannot call method '{}' on non-struct type {}",
                        method,
                        obj_value.type_name()
                    ))),
                }
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

    fn literal_to_value(&self, lit: &Literal) -> Value {
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

    fn add(left: Value, right: Value) -> RuntimeResult<Value> {
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

    fn num_op<F>(left: Value, right: Value, op: F, op_str: &str) -> RuntimeResult<Value>
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

    fn cmp<F>(left: Value, right: Value, op: F, op_str: &str) -> RuntimeResult<Value>
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
