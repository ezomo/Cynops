use crate::setting::*;

pub fn program(tokens: &mut Vec<Token>) -> Vec<Box<Node>> {
    let mut code = vec![];
    while !tokens.is_empty() {
        code.push(stmt(tokens));
    }
    code
}

pub fn stmt(tokens: &mut Vec<Token>) -> Box<Node> {
    let node = {
        if consume(Token::ctrl(ControlStructure::Return), tokens) {
            Box::new(Node::Return {
                value: expr(tokens),
            })
        } else {
            expr(tokens)
        }
    };

    if !consume(Token::stop(), tokens) {
        panic!("error");
    }
    node
}

pub fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
    assign(tokens)
}

pub fn assign(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = equality(tokens);
    if consume(Token::assign(), tokens) {
        node = Box::new(Node::Expr {
            op: ExprSymbol::Assignment,
            lhs: node,
            rhs: assign(tokens),
        });
    }
    node
}

pub fn equality(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = relational(tokens);
    loop {
        if consume(Token::comp(Comparison::Eq), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Eq),
                lhs: node,
                rhs: relational(tokens),
            });
        } else if consume(Token::comp(Comparison::Neq), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Neq),
                lhs: node,
                rhs: relational(tokens),
            });
        } else {
            return node;
        }
    }
}

pub fn relational(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = add(tokens);
    loop {
        if consume(Token::comp(Comparison::Lt), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Lt),
                lhs: node,
                rhs: add(tokens),
            });
        } else if consume(Token::comp(Comparison::Le), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Le),
                lhs: node,
                rhs: add(tokens),
            });
        } else if consume(Token::comp(Comparison::Gt), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Gt),
                lhs: node,
                rhs: add(tokens),
            });
        } else if consume(Token::comp(Comparison::Ge), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Comparison(Comparison::Ge),
                lhs: node,
                rhs: add(tokens),
            });
        } else {
            return node;
        }
    }
}

pub fn add(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(tokens);
    loop {
        if consume(Token::arith(Arithmetic::Add), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Arithmetic(Arithmetic::Add),
                lhs: node,
                rhs: mul(tokens),
            });
        } else if consume(Token::arith(Arithmetic::Sub), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Arithmetic(Arithmetic::Sub),
                lhs: node,
                rhs: mul(tokens),
            });
        } else {
            return node;
        }
    }
}

pub fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = unary(tokens);

    loop {
        if consume(Token::arith(Arithmetic::Mul), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Arithmetic(Arithmetic::Mul),
                lhs: node,
                rhs: unary(tokens),
            });
        } else if consume(Token::arith(Arithmetic::Div), tokens) {
            node = Box::new(Node::Expr {
                op: ExprSymbol::Arithmetic(Arithmetic::Div),
                lhs: node,
                rhs: unary(tokens),
            });
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
        return Box::new(Node::Expr {
            op: ExprSymbol::Arithmetic(Arithmetic::Sub),
            lhs: Box::new(Node::Value(Value::Number(0))),
            rhs: primary(tokens),
        });
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
    // そうでなければ数値か変数のはず

    return Box::new(Node::Value(consume_atom(tokens)));
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
        panic!("Expected a Token::Value, found something else.");
    }
}
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
    println!("{:?}", tokenize("return;"))
}
