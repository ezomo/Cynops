use std::{env, fs, process};

use crate::codegen::generate_program;

mod ast;
#[allow(dead_code)]
mod ast_visualizer;
mod codegen;
mod const_eval;
mod get_type;
mod lexer;
mod parser;
mod preprocessor;
mod test_cases;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }
    let mut input = fs::read_to_string(&args[1]).unwrap();
    preprocessor::remove_comments(&mut input);
    let mut token = lexer::tokenize(&input);
    let mut session = parser::ParseSession::new();
    let program: ast::Program = parser::program(&mut session, &mut token);
    // ast_visualizer::visualize_program(&program);
    generate_program(program, &mut codegen::CodeGenStatus::new());
}
