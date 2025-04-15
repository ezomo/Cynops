use std::env;
use std::process;

/// メイン関数：引数から式を受け取り、LLVM IRを出力
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    match generate_llvm_ir(&args[1]) {
        Ok(ir) => print!("{}", ir),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

/// 式を解析し、LLVM IRを文字列として返す
fn generate_llvm_ir(expr: &str) -> Result<String, String> {
    let mut p = expr.trim();
    let mut code = String::new();

    // ヘッダ部分
    code.push_str("; ModuleID = 'main'\n");
    code.push_str("define i32 @main() {\n");

    // 最初の数値（初期値）
    let (first_value, rest) = parse_number(p)?;
    p = rest;
    let mut reg_counter = 1;
    code.push_str(&format!(
        "  %{} = add i32 0, {}\n",
        reg_counter, first_value
    ));
    let mut last_reg = reg_counter;
    reg_counter += 1;

    // 残りの演算子と数値を処理
    while !p.is_empty() {
        let ch = p.chars().next().unwrap();
        p = &p[1..];

        let (num, rest2) = parse_number(p)?;
        p = rest2;

        match ch {
            '+' => {
                code.push_str(&format!(
                    "  %{} = add i32 %{}, {}\n",
                    reg_counter, last_reg, num
                ));
            }
            '-' => {
                code.push_str(&format!(
                    "  %{} = sub i32 %{}, {}\n",
                    reg_counter, last_reg, num
                ));
            }
            _ => {
                return Err(format!("予期しない文字です: '{}'", ch));
            }
        }

        last_reg = reg_counter;
        reg_counter += 1;
    }

    // 終了処理
    code.push_str(&format!("  ret i32 %{}\n", last_reg));
    code.push_str("}\n");

    Ok(code)
}

/// 数字をパースする（先頭から数字を取り出し、残りを返す）
fn parse_number(s: &str) -> Result<(i32, &str), String> {
    let s = s.trim_start();
    let mut chars = s.chars();
    let mut len = 0;

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            len += 1;
        } else {
            break;
        }
    }

    if len == 0 {
        return Err("数値のパースに失敗しました".into());
    }

    let (num_str, rest) = s.split_at(len);
    let value = num_str.parse::<i32>().map_err(|e| e.to_string())?;
    Ok((value, rest))
}
