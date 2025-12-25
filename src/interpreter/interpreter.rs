use crate::lexer::TokenKind;
use crate::parser::ast::{Expr, Literal, Stmt};
use crate::values::{Environment, Value};

#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    // =========================================================
    // Constructor
    // =========================================================

    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    // =========================================================
    // Entry point
    // =========================================================

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    // =========================================================
    // Statements
    // =========================================================

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(expr) => {
                let _ = self.eval(expr)?;
                Ok(())
            }
        }
    }

    // =========================================================
    // Expressions
    // =========================================================

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            // -------------------------
            // Literals
            // -------------------------
            Expr::_Literal_(lit) => Ok(self.literal_to_value(lit)),

            // -------------------------
            // Grouping
            // -------------------------
            Expr::Grouping(inner) => self.eval(inner),

            // -------------------------
            // Unary expressions
            // -------------------------
            Expr::Unary { operator, right } => {
                let value = self.eval(right)?;

                match operator.kind {
                    TokenKind::Minus => match value {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err("Unary '-' expects a number".to_string()),
                    },

                    TokenKind::Bang => Ok(Value::Bool(!value.truthy())),

                    _ => Err(format!("Unsupported unary operator: {:?}", operator.kind)),
                }
            }

            // -------------------------
            // Binary expressions
            // -------------------------
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;

                match operator.kind {
                    // Arithmetic
                    TokenKind::Plus => Self::add(left_val, right_val),
                    TokenKind::Minus => Self::num_op(left_val, right_val, |a, b| a - b, "-"),
                    TokenKind::Star => Self::num_op(left_val, right_val, |a, b| a * b, "*"),
                    TokenKind::Slash => Self::num_op(left_val, right_val, |a, b| a / b, "/"),

                    // Equality
                    TokenKind::EqualEqual => Ok(Value::Bool(left_val == right_val)),
                    TokenKind::BangEqual => Ok(Value::Bool(left_val != right_val)),

                    // Comparison
                    TokenKind::Greater => Self::cmp(left_val, right_val, |a, b| a > b, ">"),
                    TokenKind::GreaterEqual => Self::cmp(left_val, right_val, |a, b| a >= b, ">="),
                    TokenKind::Less => Self::cmp(left_val, right_val, |a, b| a < b, "<"),
                    TokenKind::LessEqual => Self::cmp(left_val, right_val, |a, b| a <= b, "<="),

                    _ => Err(format!("Unsupported binary operator: {:?}", operator.kind)),
                }
            }
            Expr::Log(expr) => {
                println!("{:?}", self.eval(expr).unwrap());
                Ok(Value::Bool(true))
            }
        }
    }

    // =========================================================
    // Helpers
    // =========================================================

    fn literal_to_value(&self, literal: &Literal) -> Value {
        match literal {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Bool(*b),
            Literal::Nil => Value::Null,
        }
    }

    fn add(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            _ => Err("Operator '+' expects numbers or strings".to_string()),
        }
    }

    fn num_op(
        left: Value,
        right: Value,
        f: impl Fn(f64, f64) -> f64,
        op: &str,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
            _ => Err(format!("Operator '{op}' expects numbers")),
        }
    }

    fn cmp(
        left: Value,
        right: Value,
        f: impl Fn(f64, f64) -> bool,
        op: &str,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
            _ => Err(format!("Operator '{op}' expects numbers")),
        }
    }
}
