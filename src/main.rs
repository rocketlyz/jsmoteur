use std::env;
use std::fs;
use std::process::exit;

use lib::ast::pretty_print;
use lib::interpreter::interpret;
use lib::lexer::Scanner;
use lib::parser::Parser;

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
    let tokens = scanner.scan_tokens();
    for token in &tokens {
        println!("{}", token);
    }

    println!("\n--- ast ---");
    let mut parser = Parser::new(tokens);
    let exprs = match parser.parse() {
        Ok(exprs) => {
            for expr in &exprs {
                println!("{}", pretty_print(expr));
            }
            exprs
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    println!("\n--- values ---");
    match interpret(&exprs) {
        Ok(values) => {
            for v in values {
                println!("{}", v);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
