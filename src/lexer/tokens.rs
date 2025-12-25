use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Hash,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Greater,
    Less,
    At,
    Bang,
    Percent,
    Colon,

    // Two-character (or multi-char) tokens
    GreaterEqual,
    LessEqual,
    EqualEqual,
    BangEqual,
    MinusMinus,
    PlusPlus,
    AmpersandAmpersand,
    PipePipe,
    IdentityOperator, // ?
    Ampersand,
    ColonColon, // ::
    Arrow,     // ->
    Command,   // >>

    // Keywords and other
    Import,
    Label,
    If,
    Elif,
    Else,
    Jump,
    Unlabel,
    VisibleSoft,
    VisibleHard,
    Visibility,
    Struct,
    Return,

    // variable classes
    Define,
    Rooted,
    Eternal,

    // Structure Classes
    Private,
    Public,
    SelfKw,
    Constructor,
    New,

    // macros
    DefineMacro, // _define_
    IfDef,
    IfNDef,
    UnDef,

    // switch/enum
    Enum,
    Switch,
    Case,
    Default,
    SHIF, // choose

    // bit op keywords
    BitAnd,  // _and_
    BitOr,   // _or_
    BitXor,  // _xor_
    BitComp, // _com_
    BLShift, // _lsh_
    BRShift, // _rsh_

    // extra keywords from your JS
    _DEF_,
    Def,
    Gen,
    _TTV_,
    _DELOCK_,
    Kill,
    Lock,
    Unlock,
    Revive,
    IsAlive,

    // Literals / value-kinds
    Identifier,
    Number,
    String,
    Bool,
    Char,
    Nil,

    Log,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberLit {
    Int(i128),
    Float(f64),
    BigIntString(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Identifier(String),
    Number(NumberLit),
    String(String),
    Bool(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize, // 1-based
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            literal: None,
        }
    }

    pub fn with_literal(kind: TokenKind, lexeme: impl Into<String>, line: usize, lit: Literal) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            literal: Some(lit),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.literal {
            None => write!(f, "{:?} ('{}')", self.kind, self.lexeme),
            Some(lit) => write!(f, "{:?} {:?} ('{}')", self.kind, lit, self.lexeme),
        }
    }
}
