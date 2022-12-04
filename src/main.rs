pub mod ast;
mod error;
mod lexer;
mod parser;
mod read;
mod token;
mod traits;

mod string_spliter;
use parser::Parser;
use read::read_file;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = args.get(1).unwrap();
        let input = read_file(&path);
        let mut parser = Parser::new(&input, path);
        let parse = parser.parse();

        for e in parse {
            println!("{:#?}", e)
        }
        parser.finish();
    } else {
        eprintln!("USAGE: illusio <file>")
    }
}
