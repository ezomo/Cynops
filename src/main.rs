use std::{env, fs, process};

mod ast;
mod get_type;
// mod ast_visualizer;
// mod codegen;
mod const_eval;
mod lexer;
mod parser;
mod preprocessor;
// mod sema;
// mod test_cases;
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
    println!("{:#?}", program);
    println!("{:#?}", token);

    // visualize_program(&program);

    // sema::program(program);
    // let mut cgs = codegen::CodeGenStatus::new();
    // codegen::generate_program(program, &mut cgs);
}

// #[test]
// fn test() {
//     test_cases::run_tests("test_cases").expect("Test cases failed");
// }
