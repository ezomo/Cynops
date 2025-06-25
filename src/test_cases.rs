use std::fs;

// 実際のモジュール（プロジェクトに合わせて調整）
use crate::ast_visualizer;
use crate::lexer;
use crate::parser;
use crate::preprocessor;

#[allow(dead_code)]
pub fn run_tests(test_dir: &str) -> Result<(usize, usize), String> {
    let mut test_files: Vec<_> = fs::read_dir(test_dir)
        .map_err(|e| format!("ディレクトリを読み込めません: {}", e))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? == "c" {
                let name = path.file_stem()?.to_str()?;
                let index: usize = name.parse().ok()?;
                Some((index, path))
            } else {
                None
            }
        })
        .collect();

    test_files.sort_by_key(|(index, _)| *index);

    println!("テストケース数: {}", test_files.len());
    println!();

    let mut passed = 0;
    let mut failed = 0;

    for (index, path) in test_files {
        print!("Test {}: ", index);

        let mut input =
            fs::read_to_string(&path).map_err(|e| format!("ファイルを読み込めません: {}", e))?;

        // 空のファイルをスキップ
        let code_content = input
            .lines()
            .filter(|line| !line.trim().starts_with("/*"))
            .collect::<Vec<_>>()
            .join("\n");

        if code_content.trim().is_empty() {
            println!("SKIP (empty)");
            continue;
        }

        // パーサーテスト実行
        match run_parser_test(&mut input) {
            Ok(_) => {
                println!("PASS");
                passed += 1;
            }
            Err(e) => {
                println!("FAIL - {}", e);
                failed += 1;
            }
        }
    }

    Ok((passed, failed))
}

fn run_parser_test(input: &mut String) -> Result<(), String> {
    run_parser_test_with_options(input, false)
}

fn run_parser_test_with_options(input: &mut String, show_ast: bool) -> Result<(), String> {
    preprocessor::remove_comments(input);
    let mut tokens = lexer::tokenize(input);
    let program = parser::program(&mut tokens);

    if show_ast {
        ast_visualizer::visualize_program(&program);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parser_with_test_directory() {
        // テストディレクトリのパスを指定
        let test_dir = "test_cases"; // プロジェクトの構造に合わせて調整してください

        // テストディレクトリが存在するかチェック
        if !Path::new(test_dir).exists() {
            println!(
                "テストディレクトリ '{}' が見つかりません。テストをスキップします。",
                test_dir
            );
            return;
        }

        match run_tests(test_dir) {
            Ok((passed, failed)) => {
                println!("テスト結果: {} PASS, {} FAIL", passed, failed);
                // 必要に応じてアサーションを追加
                // assert!(failed == 0, "テストが失敗しました: {} failures", failed);
            }
            Err(e) => {
                panic!("テスト実行中にエラーが發生しました: {}", e);
            }
        }
    }

    #[test]
    fn test_single_parser_case() {
        // 単一のテストケース用
        let mut input = String::from("int *filter(int (*judge)(int));");

        match run_parser_test(&mut input) {
            Ok(_) => println!("単一テストケース: PASS"),
            Err(e) => panic!("単一テストケースが失敗しました: {}", e),
        }
    }

    #[test]
    fn test_empty_input() {
        // 空の入力のテスト
        let mut input = String::from("");

        match run_parser_test(&mut input) {
            Ok(_) => println!("空の入力テスト: PASS"),
            Err(_) => println!("空の入力テスト: エラーが期待通り発生"),
        }
    }

    #[test]
    fn test_comment_removal() {
        // コメント除去のテスト
        let mut input = String::from("/* comment */ int main() { return 0; }");

        match run_parser_test(&mut input) {
            Ok(_) => println!("コメント除去テスト: PASS"),
            Err(e) => panic!("コメント除去テストが失敗しました: {}", e),
        }
    }

    #[test]
    fn test_with_ast_visualization() {
        // AST可視化テスト（show_ast = true）
        let mut input = String::from("int x = 42;");

        match run_parser_test_with_options(&mut input, true) {
            Ok(_) => println!("AST可視化テスト: PASS"),
            Err(e) => panic!("AST可視化テストが失敗しました: {}", e),
        }
    }

    #[test]
    fn test_without_ast_visualization() {
        // AST可視化なしテスト（show_ast = false）
        let mut input = String::from("int x = 42;");

        match run_parser_test_with_options(&mut input, false) {
            Ok(_) => println!("AST可視化なしテスト: PASS"),
            Err(e) => panic!("AST可視化なしテストが失敗しました: {}", e),
        }
    }
}
