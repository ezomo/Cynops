use std::env;
use std::process;
mod ast_visualizer;
mod lexer;
mod parser;
mod symbols;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    let mut token = lexer::tokenize(&args[1].to_string());
    let program = parser::program(&mut token);

    println!("{:#?}", program);

    println!("\nAST Visualization:");
    ast_visualizer::visualize_program(&program);
}
