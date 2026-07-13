//! Recursive-descent parser (expressions + statements, Ch.6–9).
//!
//! Program: declaration* until Eof
//! declaration → varDecl | statement
//! statement   → if | while | for | printStmt | block | exprStmt
//! expression  → assignment
//! assignment  → IDENTIFIER "=" assignment | logic_or
//! logic_or    → logic_and ( "||" logic_and )*
//! logic_and   → equality ( "&&" equality )*
//! … equality → … → primary
//! for desugars to Block + While.

use crate::ast::{Expr, Literal, Stmt, VarKind};
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        let mut first_err: Option<ParseError> = None;

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
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
            None => Ok(stmts),
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kinds(&[TokenKind::Var, TokenKind::Let, TokenKind::Const]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let kind = match self.previous().kind {
            TokenKind::Var => VarKind::Var,
            TokenKind::Let => VarKind::Let,
            TokenKind::Const => VarKind::Const,
            _ => unreachable!(),
        };
        let name = self
            .consume(TokenKind::Identifier, "Expect variable name.")?
            .lexeme
            .clone();
        let initializer = if self.match_kinds(&[TokenKind::Assign]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semi, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var {
            kind,
            name,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kinds(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.match_kinds(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.match_kinds(&[TokenKind::For]) {
            return self.for_statement();
        }
        if self.check_kind(&TokenKind::BraceL) {
            self.advance();
            return Ok(Stmt::Block(self.block()?));
        }
        if self.is_console_log() {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::ParenL, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::ParenR, "Expect ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_kinds(&[TokenKind::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::ParenL, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::ParenR, "Expect ')' after while condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    /// Desugar `for` into `{ initializer; while (condition) { body; increment; } }`.
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::ParenL, "Expect '(' after 'for'.")?;

        let initializer = if self.match_kinds(&[TokenKind::Semi]) {
            None
        } else if self.match_kinds(&[TokenKind::Var, TokenKind::Let, TokenKind::Const]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check_kind(&TokenKind::Semi) {
            self.expression()?
        } else {
            Expr::Literal(Literal::Bool(true))
        };
        self.consume(TokenKind::Semi, "Expect ';' after loop condition.")?;

        let increment = if !self.check_kind(&TokenKind::ParenR) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::ParenR, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }
        Ok(body)
    }

    fn is_console_log(&self) -> bool {
        if self.current + 2 >= self.tokens.len() {
            return false;
        }
        let t0 = &self.tokens[self.current];
        let t1 = &self.tokens[self.current + 1];
        let t2 = &self.tokens[self.current + 2];
        t0.kind == TokenKind::Identifier
            && t0.lexeme == "console"
            && t1.kind == TokenKind::Dot
            && t2.kind == TokenKind::Identifier
            && t2.lexeme == "log"
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // console
        self.consume(TokenKind::Dot, "Expect '.' after console.")?;
        {
            let log = self.consume(TokenKind::Identifier, "Expect 'log'.")?;
            if log.lexeme != "log" {
                let line = log.line;
                let lexeme = log.lexeme.clone();
                return Err(ParseError {
                    message: format!("at '{}': Expect 'log'.", lexeme),
                    line,
                });
            }
        }
        self.consume(TokenKind::ParenL, "Expect '(' after console.log.")?;
        let expr = self.expression()?;
        self.consume(TokenKind::ParenR, "Expect ')' after argument.")?;
        self.consume(TokenKind::Semi, "Expect ';' after console.log.")?;
        Ok(Stmt::Print(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check_kind(&TokenKind::BraceR) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenKind::BraceR, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semi, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logic_or()?;
        if self.match_kinds(&[TokenKind::Assign]) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(self.error(self.previous(), "Invalid assignment target."));
        }
        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;
        while self.match_kinds(&[TokenKind::Or]) {
            let op = self.previous().kind.clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_kinds(&[TokenKind::And]) {
            let op = self.previous().kind.clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
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

        if self.check_kind(&TokenKind::Identifier) {
            let name = self.peek().lexeme.clone();
            self.advance();
            return Ok(Expr::Variable(name));
        }

        if self.match_kinds(&[TokenKind::ParenL]) {
            let expr = self.expression()?;
            self.consume(TokenKind::ParenR, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

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

    fn parse_src(source: &str) -> Result<Vec<Stmt>, ParseError> {
        let tokens = Scanner::new(source).scan_tokens();
        Parser::new(tokens).parse()
    }

    fn expr_of(stmt: &Stmt) -> &Expr {
        match stmt {
            Stmt::Expression(e) => e,
            _ => panic!("expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn parses_precedence_mul_over_add() {
        let stmts = parse_src("1 + 2 * 3;").unwrap();
        assert_eq!(pretty_print(expr_of(&stmts[0])), "(+ 1 (* 2 3))");
    }

    #[test]
    fn parses_unary_grouping() {
        let stmts = parse_src("-(4);").unwrap();
        assert_eq!(pretty_print(expr_of(&stmts[0])), "(- (group 4))");
    }

    #[test]
    fn parses_left_associativity() {
        let stmts = parse_src("1 + 2 + 3;").unwrap();
        assert_eq!(pretty_print(expr_of(&stmts[0])), "(+ (+ 1 2) 3)");
    }

    #[test]
    fn parses_ch6_acceptance_file() {
        let stmts = parse_src("1 + 2 * 3;\n-(4);").unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(pretty_print(expr_of(&stmts[0])), "(+ 1 (* 2 3))");
        assert_eq!(pretty_print(expr_of(&stmts[1])), "(- (group 4))");
    }

    #[test]
    fn parses_string_literal() {
        let stmts = parse_src(r#""hi";"#).unwrap();
        assert_eq!(pretty_print(expr_of(&stmts[0])), r#""hi""#);
    }

    #[test]
    fn parses_var_and_console_log() {
        let stmts = parse_src("var a = 1;\nvar b = a + 3;\nconsole.log(b);").unwrap();
        assert_eq!(stmts.len(), 3);
        match &stmts[0] {
            Stmt::Var {
                kind: VarKind::Var,
                name,
                initializer: Some(init),
            } => {
                assert_eq!(name, "a");
                assert_eq!(pretty_print(init), "1");
            }
            other => panic!("{:?}", other),
        }
        match &stmts[2] {
            Stmt::Print(e) => assert_eq!(pretty_print(e), "b"),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn parses_logical_and_or() {
        use crate::ast::pretty_print;
        let stmts = parse_src("a && b || c;").unwrap();
        assert_eq!(pretty_print(expr_of(&stmts[0])), "(or (and a b) c)");
    }

    #[test]
    fn parses_while_if() {
        use crate::ast::pretty_print_stmt;
        let stmts = parse_src("while (i < 3) { i = i + 1; }\nif (i === 3) console.log(i);")
            .unwrap();
        assert!(matches!(stmts[0], Stmt::While { .. }));
        assert!(matches!(stmts[1], Stmt::If { .. }));
        let _ = pretty_print_stmt(&stmts[0]);
    }
}
