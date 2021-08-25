use std::process::exit;
use std::{env, fs};
use lib::parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(arg) => arg,
        _ => exit({
          println!("missing file");
          1
        }),
    };
    println!("Reading File {}...\n", filename);
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    println!("{}", contents);
    parse(contents);
}
