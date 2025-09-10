// mod codegen;
use std::{env, fs, process};

mod ast;
mod codegen;
mod lexer;
mod op;
mod parser;
mod preprocessor;
mod sema;
mod test_cases;
mod token;
mod typelib;
use normalize_line_endings::normalized;
mod visualize;
use visualize::*;

use crate::sema::simplification::Session;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    process(&mut args);
}

fn process(args: &mut Vec<String>) {
    if args.len() != 3 {
        eprintln!("使い方: {} <入力ファイル> <ast|codegen>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let mode = &args[2];

    let mut input = fs::read_to_string(filename).unwrap();
    input = String::from_iter(normalized(input.chars()));

    preprocessor::remove_comments(&mut input);
    preprocessor::unescape_char_literals(&mut input);

    let mut token = lexer::tokenize(&input);
    let mut session = parser::ParseSession::new();
    let mut program: ast::Program = parser::program(&mut session, &mut token);

    match mode.as_str() {
        "ast" => {
            println!("parse");
            println!("{:#?}", program);
            program.visualize();
            sema::simplification::program(&mut program, &mut Session::new());
            println!("simplification");
            program.visualize();
            let mut session = sema::ast::Session::new();
            let new_pragram = sema::convert::program(&program, &mut session);
            println!("convert");
            new_pragram.visualize();
            let new_pragram = sema::r#type::program(&new_pragram, &mut session);
            println!("typed");
            new_pragram.unwrap().visualize();
        }
        "codegen" => {
            let mut session = sema::ast::Session::new();
            sema::simplification::program(&mut program, &mut Session::new());
            let new_pragram = sema::convert::program(&program, &mut session);
            let new_program = sema::r#type::program(&new_pragram, &mut session);
            match new_program {
                Ok(program) => {
                    codegen::generate_program(program, &mut codegen::CodeGenStatus::new());
                }
                Err(e) => {
                    eprintln!("型エラー: {}", e);
                    std::process::exit(1); // 必要なら終了
                }
            }
        }
        "both" => {
            // ast_visualizer::visualize_program(&program);
            // codegen::generate_program(program.clone(), &mut codegen::CodeGenStatus::new());
        }
        _ => {
            eprintln!("不明なモード: {}", mode);
            process::exit(1);
        }
    }
}

#[test]
fn test() {
    let mut args: Vec<String> = ["", "./test.c", "ast"]
        .iter()
        .map(|x| x.to_string())
        .collect();

    process(&mut args);
}
