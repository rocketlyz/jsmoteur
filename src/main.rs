use std::env;
use std::fs;
use std::process::exit;

use lib::lexer::Scanner;

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
    let mut scanner = Scanner::new(&contents);
    for token in scanner.scan_tokens() {
        println!("{}", token);
    }
}
