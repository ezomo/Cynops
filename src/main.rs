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

fn tokenize(string: String, tokens: &mut Vec<String>) {
    let mut stack = vec![];

    for c in string.chars() {
        if c == ' ' {
            continue;
        }

        if c.is_ascii_digit() {
            stack.push(c.to_string());
            continue;
        }

        if !stack.is_empty() {
            tokens.push(stack.join(""));
            stack.clear();
        }

        if c == '+' || c == '-' {
            tokens.push(c.to_string());
        }
    }

    if !stack.is_empty() {
        tokens.push(stack.join(""));
    }
}

/// 式を解析し、LLVM IRを文字列として返す
fn generate_llvm_ir(expr: &str) -> Result<String, String> {
    let mut tokens = vec![];
    tokenize(expr.to_string(), &mut tokens);

    if tokens.is_empty() {
        return Err("トークンが空です".to_string());
    }

    let mut code = String::new();

    // ヘッダ部分
    code.push_str("; ModuleID = 'main'\n");
    code.push_str("define i32 @main() {\n");

    let mut reg_counter = 1;

    // 最初の数字
    let first = tokens.remove(0);
    let mut last_reg = reg_counter;
    code.push_str(&format!("  %{} = add i32 0, {}\n", reg_counter, first));
    reg_counter += 1;

    // 残りの演算子と数値を処理
    while !tokens.is_empty() {
        let op = tokens.remove(0);
        if tokens.is_empty() {
            return Err("演算子の後に数値が必要です".to_string());
        }
        let rhs = tokens.remove(0);

        let ir_op = match op.as_str() {
            "+" => "add",
            "-" => "sub",
            _ => return Err(format!("未知の演算子: '{}'", op)),
        };

        code.push_str(&format!(
            "  %{} = {} i32 %{}, {}\n",
            reg_counter, ir_op, last_reg, rhs
        ));
        last_reg = reg_counter;
        reg_counter += 1;
    }

    code.push_str(&format!("  ret i32 %{}\n", last_reg));
    code.push_str("}\n");

    Ok(code)
}
