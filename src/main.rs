use std::process::exit;
use std::{env, fs};
use lib::token::Token;

fn string_to_static_str(s: String) -> &'static str {
  Box::leak(s.into_boxed_str())
}

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
    let mut t1 = Token::new("name", string_to_static_str(contents));
    while true {
      let res = t1.next_token();
      println!("{}", res.to_string());
      if res.is_empty() {
        break;
      }
    }
}
