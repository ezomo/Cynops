use crate::symbols::{BinaryOp, Expr, FunctionDef, Stmt};
use crate::symbols::{Ident, Program, TopLevel};
use crate::token::*;

pub fn program(tokens: &mut Vec<Token>) -> Program {
    let mut code = Program::new();
    while !tokens.is_empty() {
        if is_next_atom(tokens) {
            code.items
                .push(TopLevel::function_def(*function_def(tokens)));
        } else {
            code.items.push(TopLevel::Stmt(*stmt(tokens)));
        }
    }

    code
}
pub fn stmt(tokens: &mut Vec<Token>) -> Box<Stmt> {
    if consume(Token::r#return(), tokens) {
        let tmp = Stmt::r#return(expr(tokens));
        if !consume(Token::Semicolon, tokens) {
            panic!("error");
        }
        tmp
    } else if consume(Token::r#if(), tokens) {
        Stmt::r#if(
            {
                consume(Token::RParen, tokens);
                let tmp = expr(tokens);
                consume(Token::LParen, tokens);
                *tmp
            },
            *stmt(tokens),
            {
                if consume(Token::r#else(), tokens) {
                    Some(*stmt(tokens))
                } else {
                    None
                }
            },
        )
    } else if consume(Token::r#while(), tokens) {
        Stmt::r#while(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(tokens);
                consume(Token::RParen, tokens);
                *tmp
            },
            *stmt(tokens),
        )
    } else if consume(Token::r#for(), tokens) {
        consume(Token::LParen, tokens);
        Stmt::r#for(
            {
                if consume(Token::Semicolon, tokens) {
                    None
                } else {
                    let tmp = expr(tokens);
                    consume(Token::Semicolon, tokens);
                    Some(*tmp)
                }
            },
            {
                if consume(Token::Semicolon, tokens) {
                    Some(*Expr::num(0))
                } else {
                    let tmp = expr(tokens);
                    consume(Token::Semicolon, tokens);
                    Some(*tmp)
                }
            },
            {
                if consume(Token::RParen, tokens) {
                    None
                } else {
                    let tmp = expr(tokens);
                    consume(Token::RParen, tokens);
                    Some(*tmp)
                }
            },
            *stmt(tokens),
        )
    } else if consume(Token::LBrace, tokens) {
        let mut code = vec![];
        block(tokens)
    } else {
        let tmp = expr(tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("error");
        }
        Stmt::expr(*tmp)
    }
}

fn block(tokens: &mut Vec<Token>) -> Box<Stmt> {
    let mut code = vec![];

    while !consume(Token::RBrace, tokens) {
        code.push(stmt(tokens));
    }

    Stmt::block(code)
}

pub fn expr(tokens: &mut Vec<Token>) -> Box<Expr> {
    assign(tokens)
}

pub fn assign(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = equality(tokens);
    if consume(Token::Equal, tokens) {
        node = Expr::assign(node, assign(tokens));
    }
    node
}

pub fn equality(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = relational(tokens);
    loop {
        if consume(Token::EqualEqual, tokens) {
            node = Expr::binary(BinaryOp::eq(), node, relational(tokens));
        } else if consume(Token::NotEqual, tokens) {
            node = Expr::binary(BinaryOp::ne(), node, relational(tokens));
        } else {
            return node;
        }
    }
}

pub fn relational(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = add(tokens);
    loop {
        if consume(Token::Less, tokens) {
            node = Expr::binary(BinaryOp::lt(), node, add(tokens));
        } else if consume(Token::LessEqual, tokens) {
            node = Expr::binary(BinaryOp::le(), node, add(tokens));
        } else if consume(Token::Greater, tokens) {
            node = Expr::binary(BinaryOp::gt(), node, add(tokens));
        } else if consume(Token::GreaterEqual, tokens) {
            node = Expr::binary(BinaryOp::ge(), node, add(tokens));
        } else {
            return node;
        }
    }
}

pub fn add(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = mul(tokens);
    loop {
        if consume(Token::Plus, tokens) {
            node = Expr::binary(BinaryOp::add(), node, mul(tokens));
        } else if consume(Token::Minus, tokens) {
            node = Expr::binary(BinaryOp::sub(), node, mul(tokens));
        } else {
            return node;
        }
    }
}

pub fn mul(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = unary(tokens);
    loop {
        if consume(Token::Asterisk, tokens) {
            node = Expr::binary(BinaryOp::mul(), node, unary(tokens));
        } else if consume(Token::Slash, tokens) {
            node = Expr::binary(BinaryOp::div(), node, unary(tokens));
        } else {
            return node;
        }
    }
}

pub fn unary(tokens: &mut Vec<Token>) -> Box<Expr> {
    if consume(Token::Plus, tokens) {
        return unary(tokens);
    }
    if consume(Token::Minus, tokens) {
        return Expr::binary(BinaryOp::sub(), Expr::num(0), unary(tokens));
    }
    return primary(tokens);
}

pub fn primary(tokens: &mut Vec<Token>) -> Box<Expr> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Token::LParen, tokens) {
        let node = expr(tokens);
        let _ = consume(Token::RParen, tokens);
        return node;
    }
    // そうでなければ数値か変数か関数のはず

    if is_next_atom(tokens) {
        consume_atom(tokens)
    } else {
        let ind = consume_ident(tokens);
        if consume(Token::LParen, tokens) {
            let tmp = Expr::call(ind, arg_list(tokens));
            consume(Token::RParen, tokens);
            tmp
        } else {
            Expr::ident(ind)
        }
    }
}

pub fn arg_list(tokens: &mut Vec<Token>) -> Vec<Box<Expr>> {
    let mut args = Vec::new();
    if !tokens.is_empty() && *tokens.first().unwrap() != Token::RParen {
        args.push(expr(tokens));
        while consume(Token::Comma, tokens) {
            args.push(expr(tokens));
        }
    }
    args
}

pub fn consume(op: Token, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

    if *tokens.first().unwrap() != op {
        return false;
    }

    tokens.remove(0);
    return true;
}

pub fn is_next_atom(tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Num(_) | Token::Char(_));
}

pub fn is_next_ident(tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Ident(_));
}

pub fn consume_atom(tokens: &mut Vec<Token>) -> Box<Expr> {
    if tokens.is_empty() {
        panic!("Expected atom, but no tokens available");
    }

    if let Some(Token::Num(n)) = tokens.first() {
        let n = n.clone();
        tokens.remove(0);
        Expr::num(n)
    } else if let Some(Token::Char(c)) = tokens.first() {
        let c = c.clone();
        tokens.remove(0);
        Expr::char_lit(c)
    } else {
        panic!()
    }
}

pub fn consume_ident(tokens: &mut Vec<Token>) -> Ident {
    if let Some(Token::Ident(name)) = tokens.first() {
        let name = name.clone();
        tokens.remove(0);
        Ident::new(name)
    } else {
        panic!("Expected identifier, found {:?}", tokens.first());
    }
}

#[test]
fn test_program() {
    use crate::tokenize::tokenize;
    let mut a = tokenize("a(b,y){return 1;}");
    let b = program(&mut a);
    println!("{:#?}", b);
}
pub fn function_def(tokens: &mut Vec<Token>) -> Box<FunctionDef> {
    let name = consume_atom(tokens);

    consume(Token::LParen, tokens);
    let mut arguments = vec![];
    while !consume(Token::RParen, tokens) {
        arguments.push(consume_atom(tokens));
        consume(Token::Comma, tokens);
    }

    consume(Token::LBrace, tokens);
    let mut code = vec![];
    while !consume(Token::RBrace, tokens) {
        code.push(stmt(tokens));
    }
    let body = Stmt::block(code);

    Stmt::function_def(name, arguments, body)
}
