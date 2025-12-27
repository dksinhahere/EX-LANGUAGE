use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct StructMethod {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {

    StructDef {
        name: String,
        methods: Vec<StructMethod>,
    },
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

    StructInstantiation {
        struct_name: String,
        method_name: String, // typically "new"
        args: Vec<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    MemberAssign {
        object: Box<Expr>,
        member: String,
        value: Box<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    MacroCall {
        var: Vec<Expr>,
        body: Vec<Stmt>
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
