pub mod ast;
mod error;
mod lexer;
mod parser;
mod read;
mod token;
mod traits;

mod string_spliter;
mod analyzer;
use analyzer::Analyzer;
use parser::Parser;
use read::read_file;
use traits::Item;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = args.get(1).unwrap();
        let input = read_file(&path);
        let mut parser = Parser::new(&input, &path.clone());
        let parse = parser.parse();
        parser.finish();
        let mut analyzer =  Analyzer::new(parse, input, path.to_string());
        analyzer.analyze();
        analyzer.finish();
    } else {
        eprintln!("USAGE: illusio <file>")
    }
}
