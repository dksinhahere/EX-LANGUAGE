use crate::lexer::Token;


pub enum Stmt {
    Expression(Expr),
    SmartLock{
        variable:String
    },
    SmartUnlock{
        variable:String
    },
    SmartKill{
        variable:String
    },
    SmartRevive{
        variable:String
    },
    SmartConst{
        variable:String
    }
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
    Log(Box<Expr>),
    Variable{
        name:String
    },
    FunctionCall {
        args: Vec<(String, Expr)>
    },
    AllocateVariable{
        name:String,
        val:Box<Expr>
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
    Nil
}


