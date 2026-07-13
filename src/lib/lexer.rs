//! Crafting Interpreters–style scanner for JavaScript.
//!
//! Longest-match for multi-char ops (`===` > `==` > `=`). Remaining operators
//! (`++`, `>>>`, compound assign, …) — see docs/TODO-scanner-tokens.md.

use crate::symbol::{keyword_from_str, Keyword};
use crate::token::{Token, TokenKind};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }
        tokens.push(Token::new(TokenKind::Eof, "", self.line));
        tokens
    }

    fn scan_token(&mut self) -> Option<Token> {
        let c = self.advance();
        match c {
            '(' => Some(self.make_token(TokenKind::ParenL)),
            ')' => Some(self.make_token(TokenKind::ParenR)),
            '{' => Some(self.make_token(TokenKind::BraceL)),
            '}' => Some(self.make_token(TokenKind::BraceR)),
            '[' => Some(self.make_token(TokenKind::BracketL)),
            ']' => Some(self.make_token(TokenKind::BracketR)),
            ',' => Some(self.make_token(TokenKind::Comma)),
            '.' => Some(self.make_token(TokenKind::Dot)),
            ';' => Some(self.make_token(TokenKind::Semi)),
            '?' => Some(self.make_token(TokenKind::Conditional)),
            ':' => Some(self.make_token(TokenKind::Colon)),
            '%' => Some(self.make_token(TokenKind::Mod)),
            '+' => Some(self.make_token(TokenKind::Add)),
            '-' => Some(self.make_token(TokenKind::Sub)),
            '*' => Some(self.make_token(TokenKind::Mul)),
            '!' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        Some(self.make_token(TokenKind::NotEqStrict))
                    } else {
                        Some(self.make_token(TokenKind::NotEq))
                    }
                } else {
                    Some(self.make_token(TokenKind::Not))
                }
            }
            '=' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        Some(self.make_token(TokenKind::EqStrict))
                    } else {
                        Some(self.make_token(TokenKind::Eq))
                    }
                } else {
                    Some(self.make_token(TokenKind::Assign))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Some(self.make_token(TokenKind::LE))
                } else {
                    Some(self.make_token(TokenKind::LT))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Some(self.make_token(TokenKind::GE))
                } else {
                    Some(self.make_token(TokenKind::GT))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Some(self.make_token(TokenKind::And))
                } else {
                    Some(self.make_token(TokenKind::Error))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Some(self.make_token(TokenKind::Or))
                } else {
                    Some(self.make_token(TokenKind::Error))
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else if self.match_char('*') {
                    self.skip_block_comment();
                    None
                } else {
                    Some(self.make_token(TokenKind::Div))
                }
            }
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.line += 1;
                None
            }
            '"' => Some(self.string()),
            _ if is_digit(c) => Some(self.number()),
            _ if is_ident_start(c) => Some(self.identifier()),
            _ => Some(self.make_token(TokenKind::Error)),
        }
    }

    fn skip_block_comment(&mut self) {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); // *
                self.advance(); // /
                return;
            }
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            if self.peek() == '\\' {
                self.advance(); // backslash
                if !self.is_at_end() {
                    self.advance(); // escaped char (\" \\ etc.)
                }
                continue;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.make_token(TokenKind::Error);
        }

        self.advance(); // closing "
        self.make_token(TokenKind::String)
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance(); // consume '.'
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme = self.lexeme();
        let value: f64 = lexeme.parse().unwrap_or(0.0);
        Token::new(TokenKind::Number(value), lexeme, self.line)
    }

    fn identifier(&mut self) -> Token {
        while is_ident_continue(self.peek()) {
            self.advance();
        }

        let text = self.lexeme();
        let kind = match text.as_str() {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => match keyword_from_str(&text) {
                Some(Keyword::Var) => TokenKind::Var,
                Some(Keyword::Let) => TokenKind::Let,
                Some(Keyword::Const) => TokenKind::Const,
                Some(Keyword::Function) => TokenKind::Function,
                Some(Keyword::If) => TokenKind::If,
                Some(Keyword::Else) => TokenKind::Else,
                Some(Keyword::Return) => TokenKind::Return,
                Some(Keyword::While) => TokenKind::While,
                Some(Keyword::For) => TokenKind::For,
                Some(Keyword::Class) => TokenKind::Class,
                Some(Keyword::New) => TokenKind::New,
                Some(Keyword::This) => TokenKind::This,
                Some(Keyword::Super) => TokenKind::Super,
                // Remaining reserved words stay identifiers for MVP parsing,
                // but keyword_from_str still recognizes them as keywords.
                Some(_) => TokenKind::Identifier,
                None => TokenKind::Identifier,
            },
        };
        Token::new(kind, text, self.line)
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.lexeme(), self.line)
    }

    fn lexeme(&self) -> String {
        self.source[self.start..self.current].to_string()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.current += ch.len_utf8();
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }
        self.advance();
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current..].chars().next().unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        let mut chars = self.source[self.current..].chars();
        chars.next();
        chars.next().unwrap_or('\0')
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || is_digit(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_test_js() {
        let source = include_str!("../../test/test.js");
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                &TokenKind::Var,
                &TokenKind::Identifier,
                &TokenKind::Assign,
                &TokenKind::Number(1.0),
                &TokenKind::Semi,
                &TokenKind::Var,
                &TokenKind::Identifier,
                &TokenKind::Assign,
                &TokenKind::Identifier,
                &TokenKind::Add,
                &TokenKind::Number(3.0),
                &TokenKind::Semi,
                &TokenKind::Function,
                &TokenKind::Identifier,
                &TokenKind::ParenL,
                &TokenKind::Identifier,
                &TokenKind::Comma,
                &TokenKind::Identifier,
                &TokenKind::ParenR,
                &TokenKind::BraceL,
                &TokenKind::Identifier,
                &TokenKind::Dot,
                &TokenKind::Identifier,
                &TokenKind::ParenL,
                &TokenKind::Identifier,
                &TokenKind::Add,
                &TokenKind::Identifier,
                &TokenKind::ParenR,
                &TokenKind::Semi,
                &TokenKind::BraceR,
                &TokenKind::Identifier,
                &TokenKind::ParenL,
                &TokenKind::ParenR,
                &TokenKind::Semi,
                &TokenKind::Eof,
            ]
        );

        assert_eq!(tokens[1].lexeme, "a");
        assert_eq!(tokens[6].lexeme, "b");
        assert_eq!(tokens[13].lexeme, "acc");
        assert_eq!(tokens[20].lexeme, "console");
        assert_eq!(tokens[22].lexeme, "log");
        assert_eq!(tokens[30].lexeme, "acc");
    }

    #[test]
    fn skips_line_and_block_comments() {
        let source = "var a = 1; // comment\n/* block */ var b = 2;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Var,
                &TokenKind::Identifier,
                &TokenKind::Assign,
                &TokenKind::Number(1.0),
                &TokenKind::Semi,
                &TokenKind::Var,
                &TokenKind::Identifier,
                &TokenKind::Assign,
                &TokenKind::Number(2.0),
                &TokenKind::Semi,
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_string_with_escape() {
        let source = r#""hello \"world\"";"#;
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].lexeme, r#""hello \"world\"""#);
        assert_eq!(tokens[1].kind, TokenKind::Semi);
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    /// Phase 1 Ch.4 acceptance: `===` / `&&` / `!` are single tokens.
    #[test]
    fn scans_ch4_acceptance() {
        let source = "var x = 1 + 2 === 3 && !(false);";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Var,
                &TokenKind::Identifier,
                &TokenKind::Assign,
                &TokenKind::Number(1.0),
                &TokenKind::Add,
                &TokenKind::Number(2.0),
                &TokenKind::EqStrict,
                &TokenKind::Number(3.0),
                &TokenKind::And,
                &TokenKind::Not,
                &TokenKind::ParenL,
                &TokenKind::False,
                &TokenKind::ParenR,
                &TokenKind::Semi,
                &TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[6].lexeme, "===");
        assert_eq!(tokens[8].lexeme, "&&");
        assert_eq!(tokens[9].lexeme, "!");
    }

    #[test]
    fn longest_match_equality_and_comparison() {
        let source = "== === != !== < <= > >=";
        let mut scanner = Scanner::new(source);
        let kinds: Vec<TokenKind> = scanner.scan_tokens().into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Eq,
                TokenKind::EqStrict,
                TokenKind::NotEq,
                TokenKind::NotEqStrict,
                TokenKind::LT,
                TokenKind::LE,
                TokenKind::GT,
                TokenKind::GE,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_logic_mod_ternary() {
        let source = "!a && b || c % 2 ? x : y";
        let mut scanner = Scanner::new(source);
        let kinds: Vec<TokenKind> = scanner.scan_tokens().into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Not,
                TokenKind::Identifier,
                TokenKind::And,
                TokenKind::Identifier,
                TokenKind::Or,
                TokenKind::Identifier,
                TokenKind::Mod,
                TokenKind::Number(2.0),
                TokenKind::Conditional,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
    }
}
