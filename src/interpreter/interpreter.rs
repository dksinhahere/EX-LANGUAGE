use crate::interpreter::error::{RuntimeError, RuntimeErrorKind, RuntimeResult};
use crate::lexer::TokenKind;
use crate::parser::ast::{Expr, Literal, Stmt};
use crate::values::values::{Environment, Value};

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
        }
    }

    fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::_Literal_(lit) => Ok(self.literal_to_value(lit)),

            Expr::Grouping(inner) => self.eval(inner),

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

            Expr::Log(expr) => {
                let value = self.eval(expr)?;
                println!("{:?}", value);
                Ok(Value::Nil)
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
