use std::env;
use std::fs;
use std::process::exit;

use lib::ast::{pretty_print, Expr, Literal};
use lib::lexer::Scanner;
use lib::token::TokenKind;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(arg) => arg,
        None => {
            println!("missing file");
            exit(1);
        }
    };
    println!("Reading File {}...\n", filename);
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    println!("--- tokens ---");
    let mut scanner = Scanner::new(&contents);
    for token in scanner.scan_tokens() {
        println!("{}", token);
    }

    // Ch.5 Representing Code — hand-built AST until Ch.6 parser exists.
    println!("\n--- ast (demo) ---");
    let add = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Number(1.0))),
        op: TokenKind::Add,
        right: Box::new(Expr::Literal(Literal::Number(2.0))),
    };
    println!("1 + 2  =>  {}", pretty_print(&add));

    let nested = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Number(1.0))),
        op: TokenKind::Add,
        right: Box::new(Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(2.0))),
            op: TokenKind::Mul,
            right: Box::new(Expr::Literal(Literal::Number(3.0))),
        }),
    };
    println!("1 + 2 * 3  =>  {}", pretty_print(&nested));

    let unary = Expr::Unary {
        op: TokenKind::Sub,
        right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
            4.0,
        ))))),
    };
    println!("-(4)  =>  {}", pretty_print(&unary));
}
