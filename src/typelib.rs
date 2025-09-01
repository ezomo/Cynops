use crate::ast::{Array, Func, Ident, Type};
use crate::parser::ParseSession;
use crate::parser::expr;
use crate::token::Token;
/// Parse a complete C type declaration

pub fn get_type(parse_session: &mut ParseSession, tokens: &Vec<Token>) -> Type {
    consume_type(parse_session, &mut tokens.clone())
}

pub fn consume_and_extract_idents(
    session: &mut ParseSession,
    tokens: &mut Vec<Token>,
) -> (Type, Vec<Ident>) {
    let original_tokens = tokens.clone();
    let original_len = tokens.len();

    let parsed_type = consume_type(session, tokens);
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
    paren_index + 1
}

pub fn consume_type(parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Type {
    let base = base(tokens, parse_session);
    call(base, tokens, parse_session)
}

fn base(tokens: &mut Vec<Token>, parse_session: &mut ParseSession) -> Type {
    let base_type = parse_session.cast(&tokens[0]).unwrap();
    tokens.remove(0);
    p(base_type, tokens)
}

fn p(mut base_type: Type, tokens: &mut Vec<Token>) -> Type {
    // 前: 基本型の前のポインタを処理
    while is_next_token(tokens, Token::Asterisk) {
        tokens.remove(0);
        base_type = Type::Pointer(Box::new(base_type));
    }
    base_type
}

fn call(mut base_type: Type, tokens: &mut Vec<Token>, parse_session: &mut ParseSession) -> Type {
    let mut center = if is_next_ident(tokens) {
        tokens.remove(0);
        vec![]
    } else if is_next_token(tokens, Token::LParen) {
        let end = find_matching_paren(tokens);
        let mut tmp: Vec<Token> = tokens.drain(0..end).collect();
        tmp.remove(0);
        tmp.pop();
        tmp
    } else {
        vec![]
    };

    // 残るは後

    if is_next_token(tokens, Token::LParen) {
        tokens.remove(0);
        let mut param_types = vec![];
        param_types.push(consume_type(parse_session, tokens));

        while is_next_token(tokens, Token::Comma) {
            tokens.remove(0);
            param_types.push(consume_type(parse_session, tokens));
        }
        tokens.remove(0);

        base_type = Type::Func(Func {
            return_type: Box::new(base_type),
            params: param_types,
        });
    } else if is_next_token(tokens, Token::LBracket) {
        tokens.remove(0);
        let mut array_sizes: Vec<Option<usize>> = vec![];

        array_sizes.push(if !is_next_token(tokens, Token::RBracket) {
            Some(
                expr(parse_session, tokens)
                    .to_typed_expr()
                    .eval_const()
                    .clone()
                    .unwrap() as usize,
            )
        } else {
            None
        });
        tokens.remove(0);
        while is_next_token(tokens, Token::LBracket) {
            tokens.remove(0);
            array_sizes.push(if !is_next_token(tokens, Token::LBracket) {
                Some(
                    expr(parse_session, tokens)
                        .to_typed_expr()
                        .eval_const()
                        .clone()
                        .unwrap() as usize,
                )
            } else {
                None
            });
            tokens.remove(0);
        }

        for size in array_sizes.into_iter().rev() {
            base_type = Type::Array(Array {
                array_of: Box::new(base_type),
                length: size,
            });
        }
    } else {
        return base_type;
    }

    if !center.is_empty() {
        call(p(base_type, &mut center), &mut center, parse_session)
    } else {
        base_type
    }
}

fn is_next_ident(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Ident(_));
}

fn is_next_token(tokens: &[Token], token: Token) -> bool {
    if tokens.is_empty() {
        return false;
    }
    tokens[0] == token
}
