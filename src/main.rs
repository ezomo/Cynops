mod codegen;
use std::{env, fs, process};

use crate::ast_visualizer::visualize_program;

mod ast;
#[allow(dead_code)]
mod ast_visualizer;
// mod codegen;
mod const_eval;
mod lexer;
mod parser;
mod preprocessor;
mod sema;
mod test_cases;
mod token;
mod typelib;

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
    // codegen::generate_program(program, &mut codegen::CodeGenStatus::new());
    visualize_program(&program);
    // println!("{:#?}", program);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_exprs() {
        let mut input = "
        struct person1 {
            char *name;
            char sex;
            int age;
            char *add;
            char *job;
        };
        "
        .to_string();
        preprocessor::remove_comments(&mut input);
        let mut token = lexer::tokenize(&input);
        let mut session = parser::ParseSession::new();
        let a = parser::program(&mut session, &mut token);
        println!("{:#?}", a);
    }
}
