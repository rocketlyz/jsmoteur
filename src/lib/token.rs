use crate::symbol::{ value_of_keyword, value_of_annotation };

enum TokenType {
  ParenL,          // (
  ParenR,          // )
  BracketL,        // [
  BracketR,        // ]
  BraceL,          // {
  BraceR,          // }
  String,          // string
  Identifier,      // identifier
  Number,          // number
  Colon,           // :
  Comma,           // ,
  True,            // true
  False,           // false
  Null,            // null
  End,             // end
  Dot,             // .
  Assign,          // =
  Semi,            // ;
  Error            // error
}

pub struct Token {
  _type: &'static str,
  source: &'static str,
  start_pos: usize,
  end_pos: usize,
  length: usize,
}

impl Token {
  pub fn new(_type: &'static str, source: &'static str) -> Self {
    Token { _type: _type, source: source, start_pos: 0, end_pos: 0, length: source.chars().count() }
  }

  pub fn get_current_char(&self, start: usize) -> &str {
    let res = self.source;
    return &res[start..start+1]
  }

  pub fn is_empty(&self, current_char: &str) -> bool {
    return current_char.contains(char::is_whitespace);
  }
  
  pub fn next_token(&mut self) -> &str {
    let mut end_pos = self.end_pos;
    let mut current_char = self.get_current_char(end_pos);
    let res = self.source;
    let start_pos = self.start_pos;
    while !self.is_empty(current_char) && end_pos < self.length {
      current_char = self.get_current_char(end_pos);
      end_pos += 1;
    }
    self.start_pos = end_pos;
    self.end_pos = end_pos;
    return &res[start_pos..end_pos];
  }
}