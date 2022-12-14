pub mod ast;
mod error;
mod lexer;
mod parser;
mod read;
mod token;
mod traits;
mod typechecker;
mod string_spliter;
mod analyzer;
mod compiler;
mod lower;
use analyzer::Analyzer;
use lower::Lower;
use parser::Parser;
use read::read_file;
use traits::Item;
use typechecker::TypeChecker;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = args.get(1).unwrap();
        let input = read_file(&path);
        let mut parser = Parser::new(&input, &path.clone());
        let parse = parser.parse();
        parser.finish();
        let mut analyzer =  Analyzer::new(&parse, input.clone(), path.to_string());
        let analyze = analyzer.analyze();

        if  analyze{
            analyzer.finish();
        }
        else {
            let mut typechecker = TypeChecker::new(&parse, &input, &path);
            if typechecker.check(){
                typechecker.finish();
            }
            else {
                let mut lower = Lower::new(&parse, typechecker);
                lower.translate();
                lower.finish();
            }
        }
    } else {
        eprintln!("USAGE: illusio <file>")
    }
}
