// mod codegen;
use std::{env, fs, process};

mod ast;
#[allow(dead_code)]
mod ast_visualizer;
mod codegen;
mod const_eval;
mod lexer;
mod parser;
mod preprocessor;
mod sema;
mod test_cases;
mod token;
mod typelib;
use normalize_line_endings::normalized;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }
    let mut input = fs::read_to_string(&args[1]).unwrap();
    input = String::from_iter(normalized(input.chars()));
    preprocessor::remove_comments(&mut input);
    preprocessor::unescape_char_literals(&mut input);
    let mut token = lexer::tokenize(&input);
    let mut session = parser::ParseSession::new();
    let program: ast::Program = parser::program(&mut session, &mut token);
    // ast_visualizer::visualize_program(&program);
    codegen::generate_program(program.clone(), &mut codegen::CodeGenStatus::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_exprs() {
        let mut input = "
            int printf(char *, char*);

            int main(void){
                char input1[4] = {'%','s','\n','\0'};
                char input2[50] = {'N','o','w',' ','i','t',' ','h','o','l','d','s',' ','t','h','e',' ','p','o','w','e','r',' ','t','o',' ','d','e','m','o','n','s','t','r','a','t','e',' ','i','t','s',' ','m','i','g','h','t','.','\n','\0'};

                printf(&input1[0],&input2[0]);
            }

        "
        .to_string();
        preprocessor::remove_comments(&mut input);
        preprocessor::unescape_char_literals(&mut input);

        let mut token = lexer::tokenize(&input);
        let mut session = parser::ParseSession::new();
        let a = parser::program(&mut session, &mut token);

        codegen::generate_program(a.clone(), &mut codegen::CodeGenStatus::new());
        ast_visualizer::visualize_program(&a);
    }
}
