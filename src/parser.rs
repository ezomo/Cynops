use crate::symbols::{AssignOp, Block, Ident, Param, Program, TopLevel, Type, UnaryOp};
use crate::symbols::{BinaryOp, Expr, FunctionDef, PostfixOp, Stmt};
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
    let mut node = logical_or(tokens);
    if consume(Token::Equal, tokens) {
        node = Expr::assign(AssignOp::equal(), node, assign(tokens));
    } else if consume(Token::PlusEqual, tokens) {
        node = Expr::assign(AssignOp::plus_equal(), node, assign(tokens));
    } else if consume(Token::MinusEqual, tokens) {
        node = Expr::assign(AssignOp::minus_equal(), node, assign(tokens));
    } else if consume(Token::AsteriskEqual, tokens) {
        node = Expr::assign(AssignOp::asterisk_equal(), node, assign(tokens));
    } else if consume(Token::SlashEqual, tokens) {
        node = Expr::assign(AssignOp::slash_equal(), node, assign(tokens));
    } else if consume(Token::PercentEqual, tokens) {
        node = Expr::assign(AssignOp::percent_equal(), node, assign(tokens));
    } else if consume(Token::CaretEqual, tokens) {
        node = Expr::assign(AssignOp::caret_equal(), node, assign(tokens));
    } else if consume(Token::PipeEqual, tokens) {
        node = Expr::assign(AssignOp::pipe_equal(), node, assign(tokens));
    } else if consume(Token::LessLessEqual, tokens) {
        node = Expr::assign(AssignOp::less_less_equal(), node, assign(tokens));
    } else if consume(Token::GreaterGreaterEqual, tokens) {
        node = Expr::assign(AssignOp::greater_greater_equal(), node, assign(tokens));
    } else if consume(Token::AmpersandEqual, tokens) {
        node = Expr::assign(AssignOp::ampersand_equal(), node, assign(tokens));
    }
    node
}

pub fn logical_or(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = logical_and(tokens);
    loop {
        if consume(Token::PipePipe, tokens) {
            node = Expr::binary(BinaryOp::pipe_pipe(), node, logical_and(tokens));
        } else {
            return node;
        }
    }
}

pub fn logical_and(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = equality(tokens);
    loop {
        if consume(Token::AmpersandAmpersand, tokens) {
            node = Expr::binary(BinaryOp::ampersand_ampersand(), node, equality(tokens));
        } else {
            return node;
        }
    }
}

pub fn equality(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = relational(tokens);
    loop {
        if consume(Token::EqualEqual, tokens) {
            node = Expr::binary(BinaryOp::equal_equal(), node, relational(tokens));
        } else if consume(Token::NotEqual, tokens) {
            node = Expr::binary(BinaryOp::not_equal(), node, relational(tokens));
        } else {
            return node;
        }
    }
}

pub fn relational(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_or(tokens);
    loop {
        if consume(Token::Less, tokens) {
            node = Expr::binary(BinaryOp::less(), node, bitwise_or(tokens));
        } else if consume(Token::LessEqual, tokens) {
            node = Expr::binary(BinaryOp::less_equal(), node, bitwise_or(tokens));
        } else if consume(Token::Greater, tokens) {
            node = Expr::binary(BinaryOp::greater(), node, bitwise_or(tokens));
        } else if consume(Token::GreaterEqual, tokens) {
            node = Expr::binary(BinaryOp::greater_equal(), node, bitwise_or(tokens));
        } else {
            return node;
        }
    }
}

pub fn bitwise_or(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_xor(tokens);
    loop {
        if consume(Token::Pipe, tokens) {
            node = Expr::binary(BinaryOp::pipe(), node, bitwise_xor(tokens));
        } else {
            return node;
        }
    }
}

pub fn bitwise_xor(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_and(tokens);
    loop {
        if consume(Token::Caret, tokens) {
            node = Expr::binary(BinaryOp::caret(), node, bitwise_and(tokens));
        } else {
            return node;
        }
    }
}

pub fn bitwise_and(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = shift(tokens);
    loop {
        if consume(Token::Ampersand, tokens) {
            node = Expr::binary(BinaryOp::ampersand(), node, shift(tokens));
        } else {
            return node;
        }
    }
}

pub fn shift(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = add(tokens);
    loop {
        if consume(Token::LessLess, tokens) {
            node = Expr::binary(BinaryOp::less_less(), node, add(tokens));
        } else if consume(Token::GreaterGreater, tokens) {
            node = Expr::binary(BinaryOp::greater_greater(), node, add(tokens));
        } else {
            return node;
        }
    }
}

pub fn add(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = mul(tokens);
    loop {
        if consume(Token::Plus, tokens) {
            node = Expr::binary(BinaryOp::plus(), node, mul(tokens));
        } else if consume(Token::Minus, tokens) {
            node = Expr::binary(BinaryOp::minus(), node, mul(tokens));
        } else {
            return node;
        }
    }
}

pub fn mul(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = unary(tokens);
    loop {
        if consume(Token::Asterisk, tokens) {
            node = Expr::binary(BinaryOp::asterisk(), node, unary(tokens));
        } else if consume(Token::Slash, tokens) {
            node = Expr::binary(BinaryOp::slash(), node, unary(tokens));
        } else if consume(Token::Percent, tokens) {
            node = Expr::binary(BinaryOp::percent(), node, unary(tokens));
        } else {
            return node;
        }
    }
}

pub fn unary(tokens: &mut Vec<Token>) -> Box<Expr> {
    if consume(Token::Plus, tokens) {
        unary(tokens)
    } else if consume(Token::Minus, tokens) {
        Expr::unary(UnaryOp::minus(), unary(tokens))
    } else if consume(Token::Bang, tokens) {
        Expr::unary(UnaryOp::bang(), unary(tokens))
    } else if consume(Token::Tilde, tokens) {
        Expr::unary(UnaryOp::tilde(), unary(tokens))
    } else if consume(Token::Ampersand, tokens) {
        Expr::unary(UnaryOp::ampersand(), unary(tokens))
    } else if consume(Token::Asterisk, tokens) {
        Expr::unary(UnaryOp::asterisk(), unary(tokens))
    } else if consume(Token::PlusPlus, tokens) {
        Expr::unary(UnaryOp::plus_plus(), unary(tokens))
    } else if consume(Token::MinusMinus, tokens) {
        Expr::unary(UnaryOp::minus_minus(), unary(tokens))
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
