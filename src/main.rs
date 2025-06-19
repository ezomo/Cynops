use std::{env, fs, process};
mod ast_visualizer;
mod lexer;
mod parser;
mod preprocessor;
mod symbols;
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
    let program = parser::program(&mut token);

    println!("\nAST Visualization:");
    ast_visualizer::visualize_program(&program);
}
