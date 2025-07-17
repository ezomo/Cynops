use crate::ast::{Array, Func, Ident, Type};
use crate::parser::ParseSession;
use crate::token::Token;

/// Parse a complete C type declaration

pub fn parse_and_extract_idents(
    session: &mut ParseSession,
    tokens: &mut Vec<Token>,
) -> (Type, Vec<Ident>) {
    let original_tokens = tokens.clone();
    let original_len = tokens.len();

    let parsed_type = parse_type(session, tokens);
    let remaining_len = tokens.len();

    // 消費されたトークンの範囲からidentを抽出
    let mut idents = vec![];
    for i in 0..(original_len - remaining_len) {
        if let Token::Ident(name) = &original_tokens[i] {
            if !session.is_base_type(&Token::Ident(name.to_string())) {
                let id = Ident { name: name.clone() };
                idents.push(id);
            }
        }
    }

    (parsed_type, idents)
}

pub fn parse_type(session: &mut ParseSession, tokens: &mut Vec<Token>) -> Type {
    let mut base_type = if session.is_base_type(&tokens[0]) {
        session.cast(&tokens.remove(0)).unwrap()
    } else {
        panic!("Expected a base type, found: {:?}", tokens);
    };

    base_type = parse_prefix_pointers(base_type, tokens);

    parse_declarator(base_type, session, tokens)
}

fn parse_prefix_pointers(mut base_type: Type, tokens: &mut Vec<Token>) -> Type {
    // 前: 基本型の前のポインタを処理
    while !tokens.is_empty() && tokens[0] == Token::Asterisk {
        base_type = Type::Pointer(Box::new(base_type));
        tokens.remove(0);
    }
    base_type
}

fn find_matching_paren(tokens: &[Token]) -> usize {
    let mut paren_index = 0;
    let mut depth = 0;

    for (i, token) in tokens.iter().enumerate() {
        if *token == Token::LParen {
            depth += 1;
        } else if *token == Token::RParen {
            depth -= 1;
            if depth == 0 {
                paren_index = i;
                break;
            }
        }
    }
    paren_index
}

fn parse_declarator(
    mut base_type: Type,
    session: &mut ParseSession,
    tokens: &mut Vec<Token>,
) -> Type {
    if tokens.is_empty() {
        return base_type;
    }

    base_type = parse_prefix_pointers(base_type, tokens);

    // 括弧の対応を見つける
    let mut skip_index = 0;

    // 真ん中を飛ばす（括弧内の処理のため）
    if !tokens.is_empty() && tokens[0] == Token::LParen {
        // 対応する)の位置を検索
        skip_index = find_matching_paren(&tokens);
        //そこも含む
        skip_index += 1;
    } else {
        if !tokens.is_empty() && matches!(tokens[0], Token::Ident(_)) {
            tokens.remove(0);
        }
    }

    // 後: 後置演算子（配列、関数）の処理
    {
        // tokens の範囲チェックを追加
        if skip_index > tokens.len() {
            return base_type;
        }

        let mut remaining_tokens = tokens[skip_index..].to_vec();
        tokens.truncate(skip_index);

        // 配列の処理
        base_type = parse_array_declarator(base_type, &mut remaining_tokens);

        // 関数の処理
        base_type = parse_function_declarator(base_type, session, &mut remaining_tokens);

        // 残りのトークンを元のtokensに戻す
        tokens.extend(remaining_tokens);
    }

    // 中: 括弧内の再帰処理
    {
        if !tokens.is_empty() && tokens[0] == Token::LParen {
            tokens.remove(0);
            tokens.remove(tokens.len() - 1);
            return parse_declarator(base_type, session, tokens);
        }
    }

    base_type
}

fn parse_array_declarator(mut base_type: Type, remaining_tokens: &mut Vec<Token>) -> Type {
    if !remaining_tokens.is_empty() && remaining_tokens[0] == Token::LBracket {
        let mut array_sizes = vec![];

        while !remaining_tokens.is_empty() && remaining_tokens[0] == Token::LBracket {
            remaining_tokens.remove(0);
            // remaining_tokens[0] は数値トークンのはず
            let size = if let Token::NumInt(n) = remaining_tokens.remove(0) {
                n
            } else {
                panic!("Expected array size");
            };
            array_sizes.push(size as usize);
            if !remaining_tokens.is_empty() && remaining_tokens[0] == Token::RBracket {
                remaining_tokens.remove(0);
            }
        }

        for size in array_sizes.into_iter().rev() {
            base_type = Type::Array(Array {
                array_of: Box::new(base_type),
                length: size,
            });
        }
    }
    base_type
}

fn parse_function_declarator(
    mut base_type: Type,
    session: &mut ParseSession,
    remaining_tokens: &mut Vec<Token>,
) -> Type {
    if !remaining_tokens.is_empty() && remaining_tokens[0] == Token::LParen {
        let mut param_types = vec![];
        let paren_end = find_matching_paren(&remaining_tokens);
        let mut param_tokens = remaining_tokens[1..paren_end].to_vec();
        remaining_tokens.drain(0..=paren_end); // remove params and closing paren

        while !param_tokens.is_empty() {
            // パラメータ型をパース
            let param_type = parse_type(session, &mut param_tokens);
            param_types.push(param_type);
            println!("{:?}", param_tokens);
            if !param_tokens.is_empty() && param_tokens[0] == Token::Comma {
                param_tokens.remove(0);
            }
        }

        base_type = Type::Func(Func {
            return_type: Some(Box::new(base_type)),
            params: param_types,
        });
    }
    base_type
}
