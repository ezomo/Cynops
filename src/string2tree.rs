use crate::setting::*;

pub fn program(tokens: &mut Vec<Token>) -> Vec<Box<Node>> {
    let mut code = vec![];
    while !tokens.is_empty() {
        code.push(stmt(tokens));
    }
    code
}

pub fn stmt(tokens: &mut Vec<Token>) -> Box<Node> {
    let node = expr(tokens);
    if !consume(Symbol::Stop, tokens) {
        panic!("error");
    }
    node
}

pub fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
    assign(tokens)
}

pub fn assign(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = equality(tokens);
    if consume(Symbol::Assignment, tokens) {
        node = Box::new(Node::new(
            Token::Symbol(Symbol::Assignment),
            Some(node),
            Some(assign(tokens)),
        ));
    }
    node
}

pub fn equality(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = relational(tokens);
    loop {
        if consume(Symbol::Comparison(Comparison::Eq), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Eq)),
                Some(node),
                Some(relational(tokens)),
            ));
        } else if consume(Symbol::Comparison(Comparison::Neq), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Neq)),
                Some(node),
                Some(relational(tokens)),
            ));
        } else {
            return node;
        }
    }
}

pub fn relational(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = add(tokens);
    loop {
        if consume(Symbol::Comparison(Comparison::Lt), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Lt)),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Comparison(Comparison::Le), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Le)),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Comparison(Comparison::Gt), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Gt)),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Comparison(Comparison::Ge), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Comparison(Comparison::Ge)),
                Some(node),
                Some(add(tokens)),
            ));
        } else {
            return node;
        }
    }
}

pub fn add(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(tokens);
    loop {
        if consume(Symbol::Arithmetic(Arithmetic::Add), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Add)),
                Some(node),
                Some(mul(tokens)),
            ));
        } else if consume(Symbol::Arithmetic(Arithmetic::Sub), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Sub)),
                Some(node),
                Some(mul(tokens)),
            ));
        } else {
            return node;
        }
    }
}

pub fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = unary(tokens);

    loop {
        if consume(Symbol::Arithmetic(Arithmetic::Mul), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Mul)),
                Some(node),
                Some(unary(tokens)),
            ));
        } else if consume(Symbol::Arithmetic(Arithmetic::Div), tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Div)),
                Some(node),
                Some(unary(tokens)),
            ));
        } else {
            return node;
        }
    }
}

pub fn unary(tokens: &mut Vec<Token>) -> Box<Node> {
    if consume(Symbol::Arithmetic(Arithmetic::Add), tokens) {
        return primary(tokens);
    }
    if consume(Symbol::Arithmetic(Arithmetic::Sub), tokens) {
        return Box::new(Node::new(
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Sub)),
            Some(Box::new(Node::new(Token::Number(0), None, None))),
            Some(primary(tokens)),
        ));
    }
    return primary(tokens);
}

pub fn primary(tokens: &mut Vec<Token>) -> Box<Node> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Symbol::Parentheses(Parentheses::L), tokens) {
        let node = expr(tokens);
        let _ = consume(Symbol::Parentheses(Parentheses::R), tokens);
        return node;
    }
    // そうでなければ数値か変数のはず

    return Box::new(Node::new(consume_atom(tokens), None, None));
}

pub fn consume(op: Symbol, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    if !matches!(next, Token::Symbol(_)) {
        return false;
    }

    if !matches!(next, Token::Symbol(s) if *s == op) {
        return false;
    }

    tokens.remove(0);
    return true;
}

pub fn consume_atom(tokens: &mut Vec<Token>) -> Token {
    if tokens.is_empty() {
        eprintln!("error_1");
    }

    let next = tokens.first().unwrap();

    if !matches!(next, Token::Number(_) | Token::Ident(_)) {
        eprintln!("{:?}error_2", next);
    }

    let tmp = next.clone();
    tokens.remove(0);
    return tmp;
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let symbols_sorted: Vec<&str> = {
        let mut syms = Symbol::SYMBOLS.to_vec();
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
                tokens.push(Token::Number(num_str.parse().unwrap()));
                input = &input[num_len..];
                continue;
            }

            // 記号（長いものから）
            let mut matched = false;
            for &sym in &symbols_sorted {
                if input.starts_with(sym) {
                    tokens.push(Token::Symbol(Symbol::classify(&sym.to_string()).unwrap()));
                    input = &input[sym.len()..];
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }

            if matches!(first, 'a'..='z') {
                tokens.push(Token::Ident(first));
                input = &input[1..];
                continue;
            }

            panic!("Unexpected character: {}", first);
        }
    }

    tokens
}
