use crate::setting;
use setting::token::Token;

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
                let num_len = num_str.len();
                tokens.push(Token::number(num_str.parse().unwrap()));
                input = &input[num_len..];
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

            if first.is_alphabetic() {
                let can_ident =
                    |c: &char| c.is_ascii_alphabetic() || c.is_ascii_digit() || *c == '_';
                let ident_str: String = input.chars().take_while(|c| can_ident(c)).collect();
                let str_len = ident_str.len();
                tokens.push(Token::ident(ident_str.to_string()));
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
    println!("{:?}", tokenize("1,2"));
}
