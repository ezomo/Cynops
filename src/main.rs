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

#[derive(Debug, Clone)]
struct CompilerOptions {
    show_parse: bool,
    show_simplification: bool,
    show_convert: bool,
    show_typed: bool,
    show_session: bool,
    run_codegen: bool,
}

impl CompilerOptions {
    fn new() -> Self {
        Self {
            show_parse: false,
            show_simplification: false,
            show_convert: false,
            show_typed: false,
            show_session: false,
            run_codegen: false,
        }
    }

    fn from_modes(modes: &[&str]) -> Result<Self, String> {
        let mut options = Self::new();

        if modes.is_empty() {
            return Err("モードが指定されていません".to_string());
        }

        for mode in modes {
            match *mode {
                // 複合モード
                "ast" => {
                    options.show_parse = true;
                    options.show_simplification = true;
                    options.show_convert = true;
                    options.show_typed = true;
                    options.show_session = true;
                }
                "all" => {
                    options.show_parse = true;
                    options.show_simplification = true;
                    options.show_convert = true;
                    options.show_typed = true;
                    options.show_session = true;
                }
                // 個別モード
                "parse" => options.show_parse = true,
                "simplification" | "simp" => options.show_simplification = true,
                "convert" | "conv" => options.show_convert = true,
                "typed" | "type" => options.show_typed = true,
                "session" | "sess" => options.show_session = true,
                "codegen" | "code" => options.run_codegen = true,
                _ => return Err(format!("不明なモード: {}", mode)),
            }
        }

        Ok(options)
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    process(&mut args);
}

fn process(args: &mut Vec<String>) {
    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let modes: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    let options = match CompilerOptions::from_modes(&modes) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("エラー: {}", e);
            print_usage(&args[0]);
            process::exit(1);
        }
    };

    let mut input = fs::read_to_string(filename).unwrap();
    input = String::from_iter(normalized(input.chars()));

    preprocessor::remove_comments(&mut input);
    preprocessor::unescape_char_literals(&mut input);

    let mut token = lexer::tokenize(&input);
    let mut session = parser::ParseSession::new();
    let mut program: ast::Program = parser::program(&mut session, &mut token);

    // 実行順序に従って処理（順序は固定、指定されたもののみ実行）

    // 1. Parse フェーズ
    if options.show_parse {
        println!("=== Parse ===");
        program.visualize();
    }

    // 2. Simplification フェーズ（必ず実行、表示は条件付き）
    let mut simp_session = Session::new();
    sema::simplification::program(&mut program, &mut simp_session);
    if options.show_simplification {
        println!("=== Simplification ===");
        program.visualize();
    }

    // 3. Convert フェーズ（必ず実行、表示は条件付き）
    let mut sema_session = sema::ast::Session::new();
    let new_program = sema::convert::program(&program, &mut sema_session);
    if options.show_convert {
        println!("=== Convert ===");
        new_program.visualize();
    }

    // 4. Type checking フェーズ（typed/session/codegenのいずれかが指定されている場合のみ実行）
    if options.show_typed || options.show_session || options.run_codegen {
        let typed_program = sema::r#type::program(&new_program, &mut sema_session);

        match typed_program {
            Ok(typed_prog) => {
                // 5. Typed結果の表示
                if options.show_typed {
                    println!("=== Typed ===");
                    typed_prog.visualize();
                }

                // 6. Session情報の表示
                if options.show_session {
                    println!("=== Session ===");
                    sema_session.visualize();
                }

                // 7. Code generation
                if options.run_codegen {
                    println!("; === Code Generation ===");
                    codegen::generate_program(typed_prog, &mut codegen::CodeGenStatus::new());
                }
            }
            Err(e) => {
                eprintln!("型エラー: {}", e);
                if options.run_codegen {
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!(
        "使い方: {} <入力ファイル> <モード1> [モード2] [モード3] ...",
        program_name
    );
    eprintln!();
    eprintln!("利用可能なモード（指定順序に関係なく適切な実行順序で処理されます）:");
    eprintln!("  ast                 - 全ての解析フェーズを表示");
    eprintln!("  all                 - 全ての解析フェーズを表示（astと同じ）");
    eprintln!("  parse               - パース結果を表示");
    eprintln!("  simplification|simp - 簡約化結果を表示");
    eprintln!("  convert|conv        - 変換結果を表示");
    eprintln!("  typed|type          - 型チェック結果を表示");
    eprintln!("  session|sess        - セッション情報を表示");
    eprintln!("  codegen|code        - コード生成を実行");
    eprintln!();
    eprintln!("実行順序: parse → simplification → convert → typed → session → codegen");
    eprintln!();
    eprintln!("例:");
    eprintln!(
        "  {} input.c parse typed               - パースと型チェック結果を表示",
        program_name
    );
    eprintln!(
        "  {} input.c session typed parse       - 順序関係なく指定可能（parse→typed→sessionの順で実行）",
        program_name
    );
    eprintln!(
        "  {} input.c all codegen               - 全フェーズ実行＋コード生成",
        program_name
    );
    eprintln!(
        "  {} input.c codegen                   - コード生成のみ",
        program_name
    );
    eprintln!(
        "  {} input.c ast                       - 従来のastモード",
        program_name
    );
}

#[test]
fn test() {
    let mut args: Vec<String> = ["", "./test.c", "session", "parse", "typed"]
        .iter()
        .map(|x| x.to_string())
        .collect();

    process(&mut args);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_parsing() {
        // 単一モード
        let opts = CompilerOptions::from_modes(&["parse"]).unwrap();
        assert!(opts.show_parse);
        assert!(!opts.show_typed);

        // 複数モード（順序は関係ない）
        let opts = CompilerOptions::from_modes(&["session", "parse", "typed"]).unwrap();
        assert!(opts.show_parse);
        assert!(opts.show_typed);
        assert!(opts.show_session);
        assert!(!opts.run_codegen);

        // エイリアス
        let opts = CompilerOptions::from_modes(&["sess", "type", "simp"]).unwrap();
        assert!(opts.show_simplification);
        assert!(opts.show_typed);
        assert!(opts.show_session);

        // 既存のモード
        let opts = CompilerOptions::from_modes(&["ast"]).unwrap();
        assert!(opts.show_parse);
        assert!(opts.show_simplification);
        assert!(opts.show_convert);
        assert!(opts.show_typed);
        assert!(opts.show_session);

        // 重複指定（問題なし）
        let opts = CompilerOptions::from_modes(&["parse", "parse", "typed"]).unwrap();
        assert!(opts.show_parse);
        assert!(opts.show_typed);

        // エラーケース
        assert!(CompilerOptions::from_modes(&["invalid"]).is_err());
        assert!(CompilerOptions::from_modes(&[]).is_err());
    }

    #[test]
    fn test_execution_order() {
        // どの順序で指定しても、同じオプションになることを確認
        let opts1 = CompilerOptions::from_modes(&["parse", "typed", "session"]).unwrap();
        let opts2 = CompilerOptions::from_modes(&["session", "parse", "typed"]).unwrap();
        let opts3 = CompilerOptions::from_modes(&["typed", "session", "parse"]).unwrap();

        assert_eq!(opts1.show_parse, opts2.show_parse);
        assert_eq!(opts1.show_typed, opts2.show_typed);
        assert_eq!(opts1.show_session, opts2.show_session);

        assert_eq!(opts1.show_parse, opts3.show_parse);
        assert_eq!(opts1.show_typed, opts3.show_typed);
        assert_eq!(opts1.show_session, opts3.show_session);
    }
}
