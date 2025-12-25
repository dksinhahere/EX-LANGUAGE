use crate::lexer::Token;


pub enum Stmt {
    Expression(Expr)
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
    Log(Box<Expr>)
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}
