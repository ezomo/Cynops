use crate::symbols::{BinaryOp, Expr, FunctionDef, PostfixOp, Stmt};
use crate::symbols::{Block, Ident, Param, Program, TopLevel, Type, UnaryOp};
use crate::token::*;

pub fn program(tokens: &mut Vec<Token>) -> Program {
    let mut code = Program::new();
    while !tokens.is_empty() {
        if is_next_type(tokens) && is_next_fn(&tokens[1..]) {
            code.items
                .push(TopLevel::function_def(*function_def(tokens)));
        } else {
            code.items.push(TopLevel::Stmt(*stmt(tokens)));
        }
    }

    code
}

pub fn function_def(tokens: &mut Vec<Token>) -> Box<FunctionDef> {
    let ret_type = consume_type(tokens);
    let name = consume_ident(tokens);
    consume(Token::LParen, tokens);
    let params = param_list(tokens);
    println!("{:?}", tokens);
    consume(Token::RParen, tokens);
    consume(Token::LBrace, tokens);

    let body = block(tokens);

    FunctionDef::new(ret_type, name, params, *body)
}

pub fn stmt(tokens: &mut Vec<Token>) -> Box<Stmt> {
    if consume(Token::r#return(), tokens) {
        let tmp = Stmt::r#return(Some(*expr(tokens)));
        if !consume(Token::Semicolon, tokens) {
            panic!("error");
        }
        tmp
    } else if consume(Token::r#if(), tokens) {
        Stmt::r#if(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(tokens);
                consume(Token::RParen, tokens);
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
    } else if consume(Token::r#break(), tokens) {
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after break statement");
        }
        Stmt::r#break()
    } else if consume(Token::r#continue(), tokens) {
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after continue statement");
        }
        Stmt::r#continue()
    } else if consume(Token::LBrace, tokens) {
        Stmt::block(*block(tokens))
    } else if is_next_type(tokens) {
        decl(tokens)
    } else {
        let tmp = expr(tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("error");
        }
        Stmt::expr(*tmp)
    }
}

pub fn decl(tokens: &mut Vec<Token>) -> Box<Stmt> {
    let ty = consume_type(tokens);
    let name = consume_ident(tokens);
    let init = if consume(Token::Equal, tokens) {
        Some(*expr(tokens))
    } else {
        None
    };
    if !consume(Token::Semicolon, tokens) {
        panic!("error");
    }
    Stmt::decl(ty, name, init)
}

fn block(tokens: &mut Vec<Token>) -> Box<Block> {
    let mut code = vec![];

    while !consume(Token::RBrace, tokens) {
        code.push(stmt(tokens));
    }

    Block::new(code)
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
        } else if consume(Token::Percent, tokens) {
            node = Expr::binary(BinaryOp::modulo(), node, unary(tokens));
        } else {
            return node;
        }
    }
}

pub fn unary(tokens: &mut Vec<Token>) -> Box<Expr> {
    if consume(Token::Plus, tokens) {
        unary(tokens)
    } else if consume(Token::Minus, tokens) {
        Expr::unary(UnaryOp::Minus, unary(tokens))
    } else if consume(Token::Bang, tokens) {
        Expr::unary(UnaryOp::Bang, unary(tokens))
    } else if consume(Token::Tilde, tokens) {
        Expr::unary(UnaryOp::bit_not(), unary(tokens))
    } else if consume(Token::Ampersand, tokens) {
        Expr::unary(UnaryOp::Ampersand, unary(tokens))
    } else if consume(Token::Asterisk, tokens) {
        Expr::unary(UnaryOp::Asterisk, unary(tokens))
    } else if consume(Token::PlusPlus, tokens) {
        Expr::unary(UnaryOp::PlusPlus, unary(tokens))
    } else if consume(Token::MinusMinus, tokens) {
        Expr::unary(UnaryOp::MinusMinus, unary(tokens))
    } else {
        postfix(tokens)
    }
}

pub fn postfix(tokens: &mut Vec<Token>) -> Box<Expr> {
    let node = primary(tokens);
    if consume(Token::PlusPlus, tokens) {
        Expr::postfix(PostfixOp::PlusPlus, node)
    } else if consume(Token::MinusMinus, tokens) {
        Expr::postfix(PostfixOp::MinusMinus, node)
    } else {
        node
    }
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
        if is_next_fn(tokens) {
            let tmp = Expr::call(consume_ident(tokens), arg_list(tokens));
            consume(Token::RParen, tokens);
            tmp
        } else {
            Expr::ident(consume_ident(tokens))
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

pub fn param_list(tokens: &mut Vec<Token>) -> Vec<Param> {
    let mut params = Vec::new();
    if !tokens.is_empty() && *tokens.first().unwrap() != Token::RParen && is_next_type(tokens) {
        params.push(param(tokens));
        while consume(Token::Comma, tokens) {
            params.push(param(tokens));
        }
    }
    params
}

pub fn param(tokens: &mut Vec<Token>) -> Param {
    let ty = consume_type(tokens);
    let name = consume_ident(tokens);
    Param::new(ty, name)
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

pub fn is_next_atom(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Num(_) | Token::Char(_));
}

pub fn is_next_ident(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Ident(_));
}

pub fn is_next_fn(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if !is_next_ident(tokens) {
        return false;
    }
    if tokens.len() < 2 {
        return false;
    }
    let second = tokens.get(1).unwrap();
    return matches!(second, Token::LParen);
}

pub fn is_next_type(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(
        next,
        Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Void)
    );
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

pub fn consume_type(tokens: &mut Vec<Token>) -> Type {
    if tokens.is_empty() {
        panic!("Expected type, but no tokens available");
    }

    let base_type = if let Some(Token::Keyword(kw)) = tokens.first() {
        let ty = match kw {
            Keyword::Int => Type::Int,
            Keyword::Char => Type::Char,
            Keyword::Void => Type::Void,
            _ => panic!("Expected type, found {:?}", kw),
        };
        tokens.remove(0); // consume the keyword
        ty
    } else {
        panic!("Expected type, found {:?}", tokens.first());
    };

    let mut pointer_depth: usize = 0;
    while consume(Token::Asterisk, tokens) {
        pointer_depth += 1;
    }
    // ポインタの深さに応じてネスト
    let mut ty = base_type;
    for _ in 0..pointer_depth {
        ty = Type::pointer(ty);
    }

    ty
}

#[test]
fn test_program() {
    use crate::lexer::tokenize;
    let mut a = tokenize("int main() { int *p = 0; }");
    let b = program(&mut a);
    println!("{:#?}", b);
}
