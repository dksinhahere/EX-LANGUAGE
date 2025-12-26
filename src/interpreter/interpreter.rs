use crate::interpreter::error::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::lexer::TokenKind;
use crate::parser::ast::{Expr, Literal, Stmt};
use crate::values::values::{ControlFlow, Environment, Function, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
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
                    let params = label_item.2.clone();
                    let args = label_item.3.clone();
                    let body = label_item.4.clone();

                    if is_callable {
                        // Store callable label as function in environment
                        let func = Value::Function(Function {
                            name: label_name.clone(),
                            params,
                            defaults: args,
                            body,
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
        }
    }

    fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::_Literal_(lit) => Ok(self.literal_to_value(lit)),

            Expr::Grouping(inner) => self.eval(inner),

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

            Expr::Variable { name } => self.environment.get(name),

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
                        // Create new scope for function execution
                        self.environment.push_scope();

                        // Build argument map from call-site arguments
                        let mut arg_map: HashMap<String, Value> = HashMap::new();
                        for (arg_name, arg_expr) in args {
                            let arg_value = self.eval(arg_expr)?;
                            arg_map.insert(arg_name.clone(), arg_value);
                        }

                        // Map external parameter names to internal variable names
                        // params: ["name", "age"] - external names used in call
                        // defaults: ["uname", "uage"] - internal names used in function body
                        
                        for (i, external_param) in func.params.iter().enumerate() {
                            let internal_name = &func.defaults[i];

                            if let Some(arg_value) = arg_map.get(external_param) {
                                // Bind argument to internal variable name
                                self.environment.define(internal_name, arg_value.clone())?;
                            } else {
                                // Missing required parameter
                                self.environment.pop_scope();
                                return Err(RuntimeError::custom(format!(
                                    "Missing required parameter '{}' in function '{}' according to arguments",
                                    external_param, function
                                )));
                            }
                        }

                        // Execute function body
                        for stmt in &func.body {
                            self.execute(stmt)?;
                        }

                        // Pop scope
                        self.environment.pop_scope();

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
