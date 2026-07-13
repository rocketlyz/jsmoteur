//! Recursive-descent expression parser (Crafting Interpreters Ch.6 / book Ch.4).
//!
//! Grammar (high → low binding via call tower):
//!   expression  → equality
//!   equality    → comparison ( ( == | != | === | !== ) comparison )*
//!   comparison  → term ( ( < | <= | > | >= ) term )*
//!   term        → factor ( ( + | - ) factor )*
//!   factor      → unary ( ( * | / | % ) unary )*
//!   unary       → ( ! | - ) unary | primary
//!   primary     → Number | String | true | false | null | "(" expression ")"
//!
//! Program (Ch.6 only): ( expression ";" )* until Eof. Stmt arrives in Ch.8.

use crate::ast::{Expr, Literal};
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    /// Parse zero or more `expression ';'` until EOF.
    /// On error: synchronize and keep going; return first error if any occurred.
    pub fn parse(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = Vec::new();
        let mut first_err: Option<ParseError> = None;

        while !self.is_at_end() {
            match self.expression_statement() {
                Ok(expr) => exprs.push(expr),
                Err(e) => {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                    self.synchronize();
                }
            }
        }

        match first_err {
            Some(e) => Err(e),
            None => Ok(exprs),
        }
    }

    fn expression_statement(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semi, "Expect ';' after expression.")?;
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_kinds(&[
            TokenKind::Eq,
            TokenKind::NotEq,
            TokenKind::EqStrict,
            TokenKind::NotEqStrict,
        ]) {
            let op = self.previous().kind.clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_kinds(&[
            TokenKind::LT,
            TokenKind::LE,
            TokenKind::GT,
            TokenKind::GE,
        ]) {
            let op = self.previous().kind.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_kinds(&[TokenKind::Add, TokenKind::Sub]) {
            let op = self.previous().kind.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_kinds(&[TokenKind::Mul, TokenKind::Div, TokenKind::Mod]) {
            let op = self.previous().kind.clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_kinds(&[TokenKind::Not, TokenKind::Sub]) {
            let op = self.previous().kind.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_kinds(&[TokenKind::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.match_kinds(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.match_kinds(&[TokenKind::Null]) {
            return Ok(Expr::Literal(Literal::Null));
        }

        if let TokenKind::Number(n) = self.peek().kind {
            self.advance();
            return Ok(Expr::Literal(Literal::Number(n)));
        }

        if self.check_kind(&TokenKind::String) {
            let lexeme = self.peek().lexeme.clone();
            self.advance();
            let inner = strip_string_lexeme(&lexeme);
            return Ok(Expr::Literal(Literal::String(inner)));
        }

        if self.match_kinds(&[TokenKind::ParenL]) {
            let expr = self.expression()?;
            self.consume(TokenKind::ParenR, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    /// Discard tokens until a statement boundary (book Panic Mode).
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semi {
                return;
            }
            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Function
                | TokenKind::Var
                | TokenKind::Let
                | TokenKind::Const
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn match_kinds(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check_kind(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check_kind(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        match (kind, &self.peek().kind) {
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ParseError> {
        if self.check_kind(&kind) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), message))
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        let msg = if token.kind == TokenKind::Eof {
            format!("at end: {}", message)
        } else {
            format!("at '{}': {}", token.lexeme, message)
        };
        ParseError {
            message: msg,
            line: token.line,
        }
    }
}

fn strip_string_lexeme(lexeme: &str) -> String {
    let bytes = lexeme.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        lexeme[1..lexeme.len() - 1].to_string()
    } else {
        lexeme.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::pretty_print;
    use crate::lexer::Scanner;

    fn parse_src(source: &str) -> Result<Vec<Expr>, ParseError> {
        let tokens = Scanner::new(source).scan_tokens();
        Parser::new(tokens).parse()
    }

    #[test]
    fn parses_precedence_mul_over_add() {
        let exprs = parse_src("1 + 2 * 3;").unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(pretty_print(&exprs[0]), "(+ 1 (* 2 3))");
    }

    #[test]
    fn parses_unary_grouping() {
        let exprs = parse_src("-(4);").unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(pretty_print(&exprs[0]), "(- (group 4))");
    }

    #[test]
    fn parses_left_associativity() {
        let exprs = parse_src("1 + 2 + 3;").unwrap();
        assert_eq!(pretty_print(&exprs[0]), "(+ (+ 1 2) 3)");
    }

    #[test]
    fn parses_ch6_acceptance_file() {
        let exprs = parse_src("1 + 2 * 3;\n-(4);").unwrap();
        assert_eq!(exprs.len(), 2);
        assert_eq!(pretty_print(&exprs[0]), "(+ 1 (* 2 3))");
        assert_eq!(pretty_print(&exprs[1]), "(- (group 4))");
    }

    #[test]
    fn parses_string_literal() {
        let exprs = parse_src(r#""hi";"#).unwrap();
        assert_eq!(pretty_print(&exprs[0]), r#""hi""#);
    }
}
