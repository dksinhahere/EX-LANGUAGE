use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeErrorKind {
    // Variable errors
    UndefinedVariable(String),
    VariableAlreadyDefined(String),
    CannotRedefineConstant(String),
    CannotRedefineSmartLocked(String),
    CannotReassignConstant(String),
    CannotReassignSmartLocked(String),
    CannotDeleteConstant(String),
    CannotDeleteSmartLocked(String),
    CannotDeleteUndefined(String),
    
    // Type errors
    TypeMismatch {
        expected: String,
        got: String,
        operation: String,
    },
    InvalidUnaryOperation {
        operator: String,
        operand_type: String,
    },
    InvalidBinaryOperation {
        operator: String,
        left_type: String,
        right_type: String,
    },
    
    // Arithmetic errors
    DivisionByZero,
    IntegerOverflow,
    InvalidNumberFormat(String),
    
    // Function/expression errors
    UnsupportedExpression(String),
    UnsupportedStatement(String),
    InvalidFunctionCall(String),
    WrongNumberOfArguments {
        expected: usize,
        got: usize,
    },
    
    // Smart lock errors
    VariableNotFound(String),
    SmartLockFailed(String),
    SmartUnlockFailed(String),
    SmartKillFailed(String),
    SmartReviveFailed(String),
    SmartConstFailed(String),
    
    // General errors
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        Self {
            kind,
            line: None,
            column: None,
            context: None,
        }
    }

    pub fn with_location(kind: RuntimeErrorKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            line: Some(line),
            column: Some(column),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    // Convenient constructors for common errors
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::UndefinedVariable(name.into()))
    }

    pub fn cannot_redefine_constant(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotRedefineConstant(name.into()))
    }

    pub fn cannot_redefine_smart_locked(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotRedefineSmartLocked(name.into()))
    }

    pub fn cannot_reassign_constant(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotReassignConstant(name.into()))
    }

    pub fn cannot_reassign_smart_locked(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotReassignSmartLocked(name.into()))
    }

    pub fn cannot_delete_constant(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotDeleteConstant(name.into()))
    }

    pub fn cannot_delete_smart_locked(name: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::CannotDeleteSmartLocked(name.into()))
    }

    pub fn type_mismatch(expected: impl Into<String>, got: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::TypeMismatch {
            expected: expected.into(),
            got: got.into(),
            operation: operation.into(),
        })
    }

    pub fn invalid_binary_op(operator: impl Into<String>, left_type: impl Into<String>, right_type: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::InvalidBinaryOperation {
            operator: operator.into(),
            left_type: left_type.into(),
            right_type: right_type.into(),
        })
    }

    pub fn division_by_zero() -> Self {
        Self::new(RuntimeErrorKind::DivisionByZero)
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::new(RuntimeErrorKind::Custom(message.into()))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Location info
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(f, "[line {}:{}] ", line, column)?;
        }

        // Error kind
        write!(f, "Runtime Error: ")?;
        
        match &self.kind {
            RuntimeErrorKind::UndefinedVariable(name) => {
                write!(f, "Undefined variable '{}'", name)?;
            }
            RuntimeErrorKind::VariableAlreadyDefined(name) => {
                write!(f, "Variable '{}' is already defined", name)?;
            }
            RuntimeErrorKind::CannotRedefineConstant(name) => {
                write!(f, "Cannot redefine constant variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotRedefineSmartLocked(name) => {
                write!(f, "Cannot redefine smart-locked variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotReassignConstant(name) => {
                write!(f, "Cannot reassign constant variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotReassignSmartLocked(name) => {
                write!(f, "Cannot reassign smart-locked variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotDeleteConstant(name) => {
                write!(f, "Cannot delete constant variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotDeleteSmartLocked(name) => {
                write!(f, "Cannot delete smart-locked variable '{}'", name)?;
            }
            RuntimeErrorKind::CannotDeleteUndefined(name) => {
                write!(f, "Cannot delete undefined variable '{}'", name)?;
            }
            RuntimeErrorKind::TypeMismatch { expected, got, operation } => {
                write!(f, "Type mismatch in '{}': expected {}, got {}", operation, expected, got)?;
            }
            RuntimeErrorKind::InvalidUnaryOperation { operator, operand_type } => {
                write!(f, "Invalid unary operation '{}' on type {}", operator, operand_type)?;
            }
            RuntimeErrorKind::InvalidBinaryOperation { operator, left_type, right_type } => {
                write!(f, "Invalid binary operation: {} {} {}", left_type, operator, right_type)?;
            }
            RuntimeErrorKind::DivisionByZero => {
                write!(f, "Division by zero")?;
            }
            RuntimeErrorKind::IntegerOverflow => {
                write!(f, "Integer overflow")?;
            }
            RuntimeErrorKind::InvalidNumberFormat(s) => {
                write!(f, "Invalid number format: '{}'", s)?;
            }
            RuntimeErrorKind::UnsupportedExpression(expr) => {
                write!(f, "Unsupported expression: {}", expr)?;
            }
            RuntimeErrorKind::UnsupportedStatement(stmt) => {
                write!(f, "Unsupported statement: {}", stmt)?;
            }
            RuntimeErrorKind::InvalidFunctionCall(msg) => {
                write!(f, "Invalid function call: {}", msg)?;
            }
            RuntimeErrorKind::WrongNumberOfArguments { expected, got } => {
                write!(f, "Wrong number of arguments: expected {}, got {}", expected, got)?;
            }
            RuntimeErrorKind::VariableNotFound(name) => {
                write!(f, "Variable '{}' not found", name)?;
            }
            RuntimeErrorKind::SmartLockFailed(msg) => {
                write!(f, "Smart lock failed: {}", msg)?;
            }
            RuntimeErrorKind::SmartUnlockFailed(msg) => {
                write!(f, "Smart unlock failed: {}", msg)?;
            }
            RuntimeErrorKind::SmartKillFailed(msg) => {
                write!(f, "Smart kill failed: {}", msg)?;
            }
            RuntimeErrorKind::SmartReviveFailed(msg) => {
                write!(f, "Smart revive failed: {}", msg)?;
            }
            RuntimeErrorKind::SmartConstFailed(msg) => {
                write!(f, "Smart const failed: {}", msg)?;
            }
            RuntimeErrorKind::Custom(msg) => {
                write!(f, "{}", msg)?;
            }
        }

        // Context info
        if let Some(context) = &self.context {
            write!(f, "\n  Context: {}", context)?;
        }

        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

// Convenience type alias
pub type RuntimeResult<T> = Result<T, RuntimeError>;