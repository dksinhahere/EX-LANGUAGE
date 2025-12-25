use std::collections::HashMap;

use crate::lexer::{Literal as LexLiteral, Token};

/// Program root
#[derive(Debug, Clone)]
pub struct Program {
    pub visible_soft: Option<VisibleBlock>,
    pub visible_hard: Option<VisibleBlock>,
    pub labels: Vec<LabelDecl>,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleKind {
    VisibleSoft,
    VisibleHard,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VisibleBlock {
    pub kind: VisibleKind,
    pub parameter: Token,
    pub declarations: Vec<Expr>,
}

/// Labels in the program:
/// - callable label: `label name(params) [&[visibility(...)] ] [ body ]`
/// - control-flow label: `label @name [ body ]`
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LabelDecl {
    Callable(CallableLabel),
    ControlFlow(ControlFlowLabel),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CallableLabel {
    pub name: Token,
    pub parameters: Vec<Expr>,
    pub visibility: Option<Vec<Token>>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ControlFlowLabel {
    pub name: Token,
    pub body: Vec<Stmt>,
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: Token,
    pub constructor: ConstructorDecl,
    pub methods: Vec<StructMethod>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConstructorDecl {
    pub name: Token,
    pub parameters: Vec<ConstructorParam>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConstructorParam {
    SelfParam(Token),
    DefinedParam(Token),
    Ident(Token),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructMethod {
    pub access_modifier: AccessModifier,
    pub name: Token,
    pub parameters: Vec<Expr>,
    pub visibility: Option<Vec<Token>>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Private,
}

/// Enum decl members
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnumMember {
    pub name: Token,
    pub value: i64,
}

/// Switch clauses
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CaseClause {
    pub value: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DefaultClause {
    pub body: Vec<Stmt>,
}

/// Literal values stored in AST
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Null,
    Bool(bool),
    Number(LexNumber),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexNumber {
    Int(i128),
    Float(f64),
    Big(String),
}

impl From<&LexLiteral> for LiteralValue {
    fn from(l: &LexLiteral) -> Self {
        match l {
            LexLiteral::Bool(b) => LiteralValue::Bool(*b),
            LexLiteral::String(s) => LiteralValue::String(s.clone()),
            LexLiteral::Char(c) => LiteralValue::Char(*c),
            LexLiteral::Number(n) => match n {
                crate::lexer::tokens::NumberLit::Int(v) => LiteralValue::Number(LexNumber::Int(*v)),
                crate::lexer::tokens::NumberLit::Float(v) => {
                    LiteralValue::Number(LexNumber::Float(*v))
                }
                crate::lexer::tokens::NumberLit::BigIntString(s) => {
                    LiteralValue::Number(LexNumber::Big(s.clone()))
                }
            },
            // identifier literal isn't used as Expr::Literal in this AST
            LexLiteral::Identifier(_) => LiteralValue::Null,
        }
    }
}

// =======================
// Expr
// =======================

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Log {
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },

    // Label call: func(x=..., y=...)
    LabelCall {
        name: Token,
        args: HashMap<String, Expr>,
    },

    // `>>` command expression
    Command {
        parts: Vec<Expr>,
    },

    // define identifier
    Define {
        variable: Token,
    },

    // new ::Ident(...)
    ObjectCreation {
        name: Token,
        args: HashMap<String, Expr>,
    },

    IndexAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        property: Token,
    },
    ObjectMethodCall {
        object: Box<Expr>,
        method: Token,
        args: HashMap<String, Expr>,
    },
    Postfix {
        operator: Token,
        operand: Box<Expr>,
    },

    EnumAccess {
        enum_name: Box<Expr>,
        variant: Token,
    },

    ListLiteral {
        elements: Vec<Expr>,
    },
    DictLiteral {
        entries: Vec<(Expr, Expr)>,
    },

    // choose condition ? if : else
    ShifChoose {
        condition: Box<Expr>,
        choose_block: Box<Expr>,
        else_block: Box<Expr>,
    },

    // bit ops (as in your JS AST)
    BitAnd {
        n1: Token,
        n2: Token,
    },
    BitOr {
        n1: Token,
        n2: Token,
    },
    BitXor {
        n1: Token,
        n2: Token,
    },
    BitComp {
        n1: Token,
    },
    BitLShift {
        n1: Token,
        n2: Token,
    },
    BitRShift {
        n1: Token,
        n2: Token,
    },

    // special default variable / deadlock / ttv accesses
    DefaultVariableDefaultAccess {
        variable: Token,
    },
    DefaultVariableGeneralAccess {
        variable: Token,
    },
    DeadlockLock {
        variable: Token,
    },
    DeadlockUnlock {
        variable: Token,
    },
    DeadlockKill {
        variable: Token,
    },
    DeadlockRevive {
        variable: Token,
    },
    DeadlockIsAlive {
        variable: Token,
    },
    AccessTtv {
        variable: Token,
        history: Box<Expr>,
    },
}

// =======================
// Stmt
// =======================

#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },

    StructDecl {
        name: Token,
        constructor: ConstructorDecl,
        methods: Vec<StructMethod>,
    },

    Var {
        name: Token,
        initializer: Expr,
    },

    Eternal {
        variable: Token,
        value: Option<Expr>,
    },
    Rooted {
        variable: Token,
        value: Option<Expr>,
    },

    Return {
        expression: Expr,
    },
    UnlabelStatement {
        value: Option<Expr>,
    },

    Jump {
        target: Token,
    },

    EnumDecl {
        name: Token,
        members: Vec<EnumMember>,
    },
    SwitchStmt {
        discriminant: Expr,
        cases: Vec<CaseClause>,
        default_case: Option<DefaultClause>,
    },

    DefaultVariable {
        variable_name: Token,
        default_value: Expr,
        mutable_value: Expr,
    },

    TTv {
        var_name: Token,
        var_value: Expr,
    },
    Delock {
        var_name: Token,
        var_value: Expr,
    },

    // Macro directives / blocks
    IfDef {
        name: Token,
        body: Vec<Stmt>,
    },
    IfNDef {
        name: Token,
        body: Vec<Stmt>,
    },
    Undef {
        macros: Vec<Token>,
    },
}