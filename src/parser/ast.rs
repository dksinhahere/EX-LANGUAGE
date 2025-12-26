use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    SmartLock {
        variable: String,
    },
    SmartUnlock {
        variable: String,
    },
    SmartKill {
        variable: String,
    },
    SmartRevive {
        variable: String,
    },
    SmartConst {
        variable: String,
    },
    Label {
        _label_: Vec<(String, bool, Vec<String>, Vec<String>, Vec<String>, Vec<Stmt>)>,
    },
    Visible {
        _name_:String,
        _block_: Vec<(String, Expr)>
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    Jump {
        jump: String,
    },
    Pass,
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    _Literal_(Literal),
    Grouping(Box<Expr>),
    Print(Box<Expr>),
    Variable {
        name: String,
    },
    FunctionCall {
        function: String,
        args: Vec<(String, Expr)>,
    },
    AllocateVariable {
        name: String,
        val: Box<Expr>,
    },

    Iterable {
        value: Vec<i128>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i128),
    Float(f64),
    BigInt(String),
    String(String),
    Bool(bool),
    Char(char),
    Nil,
}
