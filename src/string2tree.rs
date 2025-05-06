use crate::setting::node::Node;
use crate::setting::token::*;
use crate::setting::token::{ControlStructure, Token, Value};

pub fn program(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut code = vec![];
    while !tokens.is_empty() {
        code.push(stmt(tokens));
    }
    Node::program(code)
}

pub fn stmt(tokens: &mut Vec<Token>) -> Box<Node> {
    let node = {
        if consume(Token::ctrl(ControlStructure::Return), tokens) {
            let tmp = Node::r#return(expr(tokens));
            if !consume(Token::stop(), tokens) {
                panic!("error");
            }
            tmp
        } else if consume(Token::ctrl(ControlStructure::If), tokens) {
            Node::r#if(
                {
                    consume(Token::paren(Parentheses::L), tokens);
                    let tmp = expr(tokens);
                    consume(Token::paren(Parentheses::R), tokens);
                    tmp
                },
                stmt(tokens),
                {
                    if consume(Token::ctrl(ControlStructure::Else), tokens) {
                        Some(stmt(tokens))
                    } else {
                        None
                    }
                },
            )
        } else if consume(Token::ctrl(ControlStructure::While), tokens) {
            Node::r#while(
                {
                    consume(Token::paren(Parentheses::L), tokens);
                    let tmp = expr(tokens);
                    consume(Token::paren(Parentheses::R), tokens);
                    tmp
                },
                stmt(tokens),
            )
        } else if consume(Token::ctrl(ControlStructure::For), tokens) {
            consume(Token::paren(Parentheses::L), tokens);
            Node::r#for(
                {
                    if consume(Token::stop(), tokens) {
                        None
                    } else {
                        let tmp = Some(expr(tokens));
                        consume(Token::stop(), tokens);
                        tmp
                    }
                },
                {
                    if consume(Token::stop(), tokens) {
                        Node::value(Value::Number(1))
                    } else {
                        let tmp = expr(tokens);
                        consume(Token::stop(), tokens);
                        tmp
                    }
                },
                {
                    if consume(Token::paren(Parentheses::R), tokens) {
                        None
                    } else {
                        let tmp = Some(expr(tokens));
                        consume(Token::paren(Parentheses::R), tokens);
                        tmp
                    }
                },
                stmt(tokens),
            )
        } else if consume(Token::block(BlockBrace::L), tokens) {
            let mut code = vec![];
            while !consume(Token::block(BlockBrace::R), tokens) {
                code.push(stmt(tokens));
            }
            Node::program(code)
        } else {
            let tmp = expr(tokens);
            if !consume(Token::stop(), tokens) {
                panic!("error");
            }
            tmp
        }
    };

    node
}

pub fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
    assign(tokens)
}

pub fn assign(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = equality(tokens);
    if consume(Token::assign(), tokens) {
        node = Node::expr(ExprSymbol::Assignment, node, assign(tokens))
    }
    node
}

pub fn equality(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = relational(tokens);
    loop {
        if consume(Token::comp(Comparison::Eq), tokens) {
            node = Node::expr(
                ExprSymbol::Comparison(Comparison::Eq),
                node,
                relational(tokens),
            );
        } else if consume(Token::comp(Comparison::Neq), tokens) {
            node = Node::expr(
                ExprSymbol::Comparison(Comparison::Neq),
                node,
                relational(tokens),
            );
        } else {
            return node;
        }
    }
}

pub fn relational(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = add(tokens);
    loop {
        if consume(Token::comp(Comparison::Lt), tokens) {
            node = Node::expr(ExprSymbol::Comparison(Comparison::Lt), node, add(tokens));
        } else if consume(Token::comp(Comparison::Le), tokens) {
            node = Node::expr(ExprSymbol::Comparison(Comparison::Le), node, add(tokens));
        } else if consume(Token::comp(Comparison::Gt), tokens) {
            node = Node::expr(ExprSymbol::Comparison(Comparison::Gt), node, add(tokens));
        } else if consume(Token::comp(Comparison::Ge), tokens) {
            node = Node::expr(ExprSymbol::Comparison(Comparison::Ge), node, add(tokens));
        } else {
            return node;
        }
    }
}

pub fn add(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(tokens);
    loop {
        if consume(Token::arith(Arithmetic::Add), tokens) {
            node = Node::expr(ExprSymbol::Arithmetic(Arithmetic::Add), node, mul(tokens));
        } else if consume(Token::arith(Arithmetic::Sub), tokens) {
            node = Node::expr(ExprSymbol::Arithmetic(Arithmetic::Sub), node, mul(tokens));
        } else {
            return node;
        }
    }
}

pub fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = unary(tokens);

    loop {
        if consume(Token::arith(Arithmetic::Mul), tokens) {
            node = Node::expr(ExprSymbol::Arithmetic(Arithmetic::Mul), node, unary(tokens));
        } else if consume(Token::arith(Arithmetic::Div), tokens) {
            node = Node::expr(ExprSymbol::Arithmetic(Arithmetic::Div), node, unary(tokens));
        } else {
            return node;
        }
    }
}

pub fn unary(tokens: &mut Vec<Token>) -> Box<Node> {
    if consume(Token::arith(Arithmetic::Add), tokens) {
        return primary(tokens);
    }
    if consume(Token::arith(Arithmetic::Sub), tokens) {
        return Node::expr(
            ExprSymbol::Arithmetic(Arithmetic::Sub),
            Node::value(Value::Number(0)),
            primary(tokens),
        );
    }
    return primary(tokens);
}

pub fn primary(tokens: &mut Vec<Token>) -> Box<Node> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Token::paren(Parentheses::L), tokens) {
        let node = expr(tokens);
        let _ = consume(Token::paren(Parentheses::R), tokens);
        return node;
    }
    // そうでなければ数値か変数か関数のはず

    let value = Node::value(consume_atom(tokens));

    if consume(Token::paren(Parentheses::L), tokens) {
        let tmp = Node::call(value, arg_list(tokens));
        consume(Token::paren(Parentheses::R), tokens);
        tmp
    } else {
        value
    }
}

pub fn arg_list(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = vec![expr(tokens)];
    loop {
        if consume(Token::comma(), tokens) {
            node.push(expr(tokens));
        } else {
            return Node::program(node);
        }
    }
}

pub fn consume(op: Token, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    if matches!(next, Token::Value(_)) {
        return false;
    }

    if *next != op {
        return false;
    }

    tokens.remove(0);
    return true;
}

pub fn consume_atom(tokens: &mut Vec<Token>) -> Value {
    if tokens.is_empty() {
        eprintln!("error_1");
    }
    let next = tokens.first().unwrap();
    if let Token::Value(value) = next.clone() {
        tokens.remove(0); // 要素を削除
        return value.clone();
    } else {
        panic!("Expected a Token::Value, found {:?}.", next);
    }
}

#[test]
fn test_program() {
    use crate::tokenize::tokenize;
    let mut a = tokenize(" a(2,3);");
    let b = program(&mut a);
    println!("{:#?}", b);
}
