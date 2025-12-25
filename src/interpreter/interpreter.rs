use crate::lexer::TokenKind;
use crate::parser::ast::{Expr, LexNumber, LiteralValue, Program, Stmt};
use crate::values::{Environment, Value};

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

    // -------------------------
    // Pipeline entry
    // -------------------------
    pub fn accept(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    // -------------------------
    // Statement execution
    // -------------------------
    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements } => self.execute_block(statements),

            // IMPORTANT: your ast.rs uses "Expression" not "Expr"
            Stmt::Expression { expression } => self.execute_expression(expression),

            // TODO: handle Var/If/While/Print/etc later
            _ => Ok(()),
        }
    }

    pub fn execute_expression(&mut self, expr: &Expr) -> Result<(), String> {
        let _v = self.eval(expr)?;
        // In REPL you may want: println!("{}", Self::value_to_string(&_v));
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), String> {
        self.environment.push_scope();

        let result = (|| {
            for s in statements {
                self.execute(s)?;
            }
            Ok::<(), String>(())
        })();

        self.environment.pop_scope();
        result
    }

    // -------------------------
    // Expression evaluation
    // -------------------------
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal { value } => Ok(Self::literal_to_value(value)),

            Expr::Log { value } => {
                let v = self.eval(value)?;
                println!("{}", Self::value_to_string(&v));
                Ok(v)
            }
            Expr::Grouping { expression } => self.eval(expression),

            Expr::Variable { name } => self
                .environment
                .get(&name.lexeme)
                .ok_or_else(|| format!("Undefined variable '{}'", name.lexeme)),

            Expr::Assign { name, value } => {
                let v = self.eval(value)?;
                if self.environment.assign(&name.lexeme, v.clone()) {
                    Ok(v)
                } else {
                    Err(format!(
                        "Assign failed (undefined or constant): '{}'",
                        name.lexeme
                    ))
                }
            }

            // Prefix ops: - ! ++ --
            Expr::Unary { operator, right } => match operator.kind {
                // ---------- PREFIX ++ / -- ----------
                TokenKind::PlusPlus | TokenKind::MinusMinus => {
                    // must be a variable (so we can write back)
                    let var_name = match right.as_ref() {
                        Expr::Variable { name } => name.lexeme.clone(),
                        _ => {
                            return Err(
                                "Prefix ++/-- requires a variable operand (e.g. ++x or --x)"
                                    .to_string(),
                            )
                        }
                    };

                    let current = self
                        .environment
                        .get(&var_name)
                        .ok_or_else(|| format!("Undefined variable '{var_name}'"))?;

                    let Value::Number(n) = current else {
                        return Err(format!("Prefix {:?} requires a number", operator.kind));
                    };

                    let new_value = match operator.kind {
                        TokenKind::PlusPlus => Value::Number(n + 1.0),
                        TokenKind::MinusMinus => Value::Number(n - 1.0),
                        _ => unreachable!(),
                    };

                    if !self.environment.assign(&var_name, new_value.clone()) {
                        return Err(format!("Failed to assign to '{var_name}'"));
                    }

                    // prefix returns NEW value
                    Ok(new_value)
                }

                // ---------- NORMAL UNARY ----------
                TokenKind::Minus => {
                    let r = self.eval(right)?;
                    match r {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err("Unary '-' requires a number".to_string()),
                    }
                }

                TokenKind::Bang => {
                    let r = self.eval(right)?;
                    Ok(Value::Bool(!r.truthy()))
                }

                _ => Err(format!("Unary operator not supported: {:?}", operator.kind)),
            },

            // Postfix ops: x++ x--
            Expr::Postfix { operator, operand } => {
                let var_name = match operand.as_ref() {
                    Expr::Variable { name } => name.lexeme.clone(),
                    _ => {
                        return Err(
                            "Postfix operator requires a variable operand (like x++ or x--)"
                                .to_string(),
                        )
                    }
                };

                let current = self
                    .environment
                    .get(&var_name)
                    .ok_or_else(|| format!("Undefined variable '{var_name}'"))?;

                let Value::Number(n) = current else {
                    return Err(format!("Postfix {:?} requires a number", operator.kind));
                };

                let new_value = match operator.kind {
                    TokenKind::PlusPlus => Value::Number(n + 1.0),
                    TokenKind::MinusMinus => Value::Number(n - 1.0),
                    _ => return Err(format!("Unsupported postfix operator: {:?}", operator.kind)),
                };

                if !self.environment.assign(&var_name, new_value) {
                    return Err(format!("Failed to assign to '{var_name}'"));
                }

                // postfix returns OLD value
                Ok(Value::Number(n))
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // short-circuit for && and ||
                match operator.kind {
                    TokenKind::AmpersandAmpersand => {
                        let l = self.eval(left)?;
                        if !l.truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let r = self.eval(right)?;
                        return Ok(Value::Bool(r.truthy()));
                    }
                    TokenKind::PipePipe => {
                        let l = self.eval(left)?;
                        if l.truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let r = self.eval(right)?;
                        return Ok(Value::Bool(r.truthy()));
                    }
                    _ => {}
                }

                let l = self.eval(left)?;
                let r = self.eval(right)?;

                match operator.kind {
                    TokenKind::Plus => Self::add_values(l, r),
                    TokenKind::Minus => Self::num_bin(l, r, |a, b| a - b, "-"),
                    TokenKind::Star => Self::num_bin(l, r, |a, b| a * b, "*"),
                    TokenKind::Slash => Self::num_bin(l, r, |a, b| a / b, "/"),
                    TokenKind::Percent => Self::num_bin(l, r, |a, b| a % b, "%"),

                    TokenKind::EqualEqual => Ok(Value::Bool(l == r)),
                    TokenKind::BangEqual => Ok(Value::Bool(l != r)),

                    TokenKind::Greater => Self::num_cmp(l, r, |a, b| a > b, ">"),
                    TokenKind::GreaterEqual => Self::num_cmp(l, r, |a, b| a >= b, ">="),
                    TokenKind::Less => Self::num_cmp(l, r, |a, b| a < b, "<"),
                    TokenKind::LessEqual => Self::num_cmp(l, r, |a, b| a <= b, "<="),

                    _ => Err(format!(
                        "Binary operator not supported: {:?}",
                        operator.kind
                    )),
                }
            }

            other => Err(format!("Expression not implemented yet: {other:?}")),
        }
    }

    // -------------------------
    // Helpers (as methods)
    // -------------------------
    fn literal_to_value(l: &LiteralValue) -> Value {
        match l {
            LiteralValue::Null => Value::Null,
            LiteralValue::Bool(b) => Value::Bool(*b),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Char(c) => Value::Char(*c),
            LiteralValue::Number(n) => match n {
                LexNumber::Int(v) => Value::Number(*v as f64),
                LexNumber::Float(v) => Value::Number(*v),
                LexNumber::Big(s) => s.parse::<f64>().map(Value::Number).unwrap_or(Value::Null),
            },
        }
    }

    fn value_to_string(v: &Value) -> String {
        match v {
            Value::Null => "nil".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
        }
    }

    fn add_values(l: Value, r: Value) -> Result<Value, String> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            (Value::String(a), b) => Ok(Value::String(format!("{a}{}", Self::value_to_string(&b)))),
            (a, Value::String(b)) => {
                Ok(Value::String(format!("{}{}", Self::value_to_string(&a), b)))
            }
            _ => Err("Operator '+' requires numbers or strings".to_string()),
        }
    }

    fn num_bin(l: Value, r: Value, f: impl Fn(f64, f64) -> f64, op: &str) -> Result<Value, String> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
            _ => Err(format!("Operator '{op}' requires numbers")),
        }
    }

    fn num_cmp(
        l: Value,
        r: Value,
        f: impl Fn(f64, f64) -> bool,
        op: &str,
    ) -> Result<Value, String> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
            _ => Err(format!("Operator '{op}' requires numbers")),
        }
    }
}
