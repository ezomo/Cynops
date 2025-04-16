use std::char;
use std::env;
use std::process;
use std::usize;

#[derive(Debug, PartialEq, Clone)]
enum SYMBOLS {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    ParenL, // (
    ParenR, // )
}

impl SYMBOLS {
    fn classify(input: char) -> Option<SYMBOLS> {
        match input {
            '+' => Some(Self::Add),
            '-' => Some(Self::Sub),
            '*' => Some(Self::Mul),
            '/' => Some(Self::Div),
            '(' => Some(Self::ParenL),
            ')' => Some(Self::ParenR),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum TokenKind {
    SYMBOL, // 記号
    NUMBER, // 整数トークン
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,         // トークンの型
    value: Option<usize>,    // kindがTK_NUMの場合、その数値
    symbol: Option<SYMBOLS>, // トークン文字列
}

impl Token {
    fn new_number(value: usize) -> Self {
        Self {
            kind: TokenKind::NUMBER,
            value: Some(value),
            symbol: None,
        }
    }
    fn new_symbol(symbol: SYMBOLS) -> Self {
        Self {
            kind: TokenKind::SYMBOL,
            value: None,
            symbol: Some(symbol),
        }
    }
}

#[derive(Debug)]
enum NodeKind {
    ADD, // +
    SUB, // -
    MUL, // *
    DIV, // /
    NUM, // 整数
}

// 抽象構文木のノードの型
#[derive(Debug)]
struct Node {
    kind: NodeKind,         // ノードの型
    lhs: Option<Box<Node>>, // 左辺
    rhs: Option<Box<Node>>, // 右辺
    val: Option<usize>,     // kindがND_NUMの場合のみ使う
}

impl Node {
    fn new_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
        }
    }

    fn new_node_num(val: usize) -> Self {
        Self {
            kind: NodeKind::NUM,
            lhs: None,
            rhs: None,
            val: Some(val),
        }
    }
}

fn primary(tokens: &mut Vec<Token>) -> Box<Node> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(SYMBOLS::ParenL, tokens) {
        let node = expr(tokens);
        let _ = consume(SYMBOLS::ParenR, tokens);
        return node;
    }

    // そうでなければ数値のはず
    return Box::new(Node::new_node_num(expect_number(tokens)));
}

fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = primary(tokens);

    loop {
        if consume(SYMBOLS::Mul, tokens) {
            node = Box::new(Node::new_node(NodeKind::MUL, node, primary(tokens)));
        } else if consume(SYMBOLS::Div, tokens) {
            node = Box::new(Node::new_node(NodeKind::DIV, node, primary(tokens)));
        } else {
            return node;
        }
    }
}

fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(tokens);
    loop {
        if consume(SYMBOLS::Add, tokens) {
            node = Box::new(Node::new_node(NodeKind::ADD, node, mul(tokens)));
        } else if consume(SYMBOLS::Sub, tokens) {
            node = Box::new(Node::new_node(NodeKind::SUB, node, mul(tokens)));
        } else {
            return node;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    let tokens = tokenize(args[1].clone());
}

fn tokenize(string: String) -> Vec<Token> {
    let mut tokens = vec![];
    let mut stack = vec![];

    for c in string.chars() {
        if c == ' ' {
            continue;
        }

        if c.is_ascii_digit() {
            stack.push(c.to_string());
            continue;
        }

        if !stack.is_empty() {
            tokens.push(Token::new_number(stack.join("").parse().unwrap()));
            stack.clear();
        }
        let symbol = SYMBOLS::classify(c);
        if symbol.is_some() {
            tokens.push(Token::new_symbol(symbol.unwrap()));
        }
    }

    if !stack.is_empty() {
        tokens.push(Token::new_number(stack.join("").parse().unwrap()));
    }

    return tokens;
}

#[test]
fn test() {
    let a = "(3+3)*2";

    let mut b = tokenize(a.to_string());
    let f = expr(&mut b);
    println!("{:?}", f);
}

fn consume(op: SYMBOLS, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    if next.kind != TokenKind::SYMBOL {
        return false;
    }

    if next.symbol.clone().unwrap() != op {
        return false;
    }

    tokens.remove(0);
    return true;
}

fn expect_number(tokens: &mut Vec<Token>) -> usize {
    if tokens.is_empty() {
        eprintln!("error");
    }

    let next = tokens.first().unwrap();

    if next.kind != TokenKind::NUMBER {
        eprintln!("error");
    }
    let tmp = next.value.unwrap();
    tokens.remove(0);
    return tmp;
}
