use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Punctuation
    ParenL,
    ParenR,
    BraceL,
    BraceR,
    BracketL,
    BracketR,
    Comma,
    Dot,
    Semi,
    Colon,
    Conditional, // ?

    // Assignment / arithmetic
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison / equality (longest-match: === before == before =)
    LT,
    GT,
    LE,
    GE,
    Eq,
    NotEq,
    EqStrict,
    NotEqStrict,

    // Logic / unary
    Not,
    And,
    Or,

    // Literals
    Identifier,
    String,
    Number(f64),
    True,
    False,
    Null,

    // Keywords
    Var,
    Let,
    Const,
    Function,
    If,
    Else,
    Return,
    While,
    For,
    Class,
    New,
    This,
    Super,

    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize) -> Self {
        Token {
            kind,
            lexeme: lexeme.into(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Number(n) => write!(f, "[NUMBER {} line {}]", n, self.line),
            TokenKind::Eof => write!(f, "[EOF '' line {}]", self.line),
            TokenKind::Error => write!(f, "[ERROR '{}' line {}]", self.lexeme, self.line),
            other => write!(f, "[{:?} '{}' line {}]", other, self.lexeme, self.line),
        }
    }
}
