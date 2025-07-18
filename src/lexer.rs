use crate::token;
use token::Token;

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let symbols_sorted: Vec<&str> = {
        let mut syms: Vec<_> = Token::SYMBOLS.iter().map(|x| x.0).collect();
        syms.sort_by(|a, b| b.len().cmp(&a.len())); // 長い記号優先
        syms
    };

    let mut input = input.trim();
    while !input.is_empty() {
        input = input.trim_start();
        if let Some(first) = input.chars().next() {
            // 数字
            if first.is_ascii_digit() {
                let num_str: String = input.chars().take_while(|c| c.is_ascii_digit()).collect();
                input = &input[num_str.len()..];

                if input.starts_with('.') {
                    input = &input[1..];
                    let num_str2: String =
                        input.chars().take_while(|c| c.is_ascii_digit()).collect();
                    input = &input[num_str2.len()..];

                    tokens.push(Token::NumFloat(
                        format!("{}.{}", num_str, num_str2).parse().unwrap(),
                    ));
                } else {
                    tokens.push(Token::NumInt(num_str.parse().unwrap()));
                }

                continue;
            }

            // 記号（長いものから）
            let mut matched = false;
            for &sym in &symbols_sorted {
                if input.starts_with(sym) {
                    tokens.push(Token::classify(&sym.to_string()).unwrap());
                    input = &input[sym.len()..];
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }

            // 文字リテラル
            if input.starts_with('\'') {
                tokens.push(Token::Char(input.chars().nth(1).unwrap()));
                if input.chars().nth(2).unwrap() != '\'' {
                    panic!("error")
                }
                input = &input[3..];
                continue;
            }

            if input.starts_with('"') {
                let mut end = 1;
                let mut escaped = false;

                while end < input.len() {
                    let c = input.chars().nth(end).unwrap();

                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    }

                    end += 1;
                }

                if end >= input.len() || input.chars().nth(end).unwrap() != '"' {
                    panic!("unterminated string literal");
                }

                let content: String = input[1..end].to_string();
                tokens.push(Token::String(content));
                input = &input[end + 1..];
                continue;
            }

            // 識別子
            if first.is_alphabetic() {
                let can_ident =
                    |c: &char| c.is_ascii_alphabetic() || c.is_ascii_digit() || *c == '_';
                let ident_str: String = input.chars().take_while(|c| can_ident(c)).collect();
                let str_len = ident_str.len();
                tokens.push(Token::Ident(ident_str.to_string()));
                input = &input[str_len..];
                continue;
            }

            panic!("Unexpected character: {}", first);
        }
    }

    tokens
}
#[test]
fn test_tokenize() {
    println!("{:?}", tokenize("a=1"));
}
