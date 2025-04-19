use std::env;
use std::process;
use std::usize;

#[derive(Debug, PartialEq, Clone)]
enum Symbol {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    ParenL, // (
    ParenR, // )
    Eq,     // ==
    Neq,    // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=
}
impl Symbol {
    const SYMBOLS: [&str; 12] = [
        "+", "-", "*", "/", "(", ")", "==", "!=", "<", "<=", ">", ">=",
    ];
    fn classify(input: &str) -> Option<Self> {
        match input {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "(" => Some(Self::ParenL),
            ")" => Some(Self::ParenR),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Neq),
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Le),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Ge),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(usize),  // 数値リテラル
    Symbol(Symbol), // 記号トークン
}

// 抽象構文木のノードの型
#[derive(Debug, PartialEq, Clone)]
struct Node {
    token: Token,           // ノードの型
    lhs: Option<Box<Node>>, // 左辺
    rhs: Option<Box<Node>>, // 右辺
}
impl Node {
    fn new(token: Token, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Self {
            token,
            lhs: lhs,
            rhs: rhs,
        }
    }
}

// for debug
impl Node {
    // 既存のメソッド等はそのまま

    // 抽象構文木を図示する関数
    #[allow(dead_code)]
    fn visualize(&self, prefix: &str, is_last: bool) {
        let display_prefix = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last { "    " } else { "│   " };

        // ノードの値を表示
        let node_value = match &self.token {
            Token::Number(n) => format!("Number({})", n),
            Token::Symbol(s) => match s {
                Symbol::Add => "Symbol(+)".to_string(),
                Symbol::Sub => "Symbol(-)".to_string(),
                Symbol::Mul => "Symbol(*)".to_string(),
                Symbol::Div => "Symbol(/)".to_string(),
                Symbol::ParenL => "Symbol(()".to_string(),
                Symbol::ParenR => "Symbol())".to_string(),
                Symbol::Eq => "Symbol(==)".to_string(),
                Symbol::Neq => "Symbol(!=)".to_string(),
                Symbol::Lt => "Symbol(<)".to_string(),
                Symbol::Le => "Symbol(<=)".to_string(),
                Symbol::Gt => "Symbol(>)".to_string(),
                Symbol::Ge => "Symbol(>=)".to_string(),
            },
        };

        println!("{}{}{}", prefix, display_prefix, node_value);

        // 左の子ノードを表示
        if let Some(lhs) = &self.lhs {
            let has_right = self.rhs.is_some();
            lhs.visualize(&format!("{}{}", prefix, next_prefix), !has_right);
        }

        // 右の子ノードを表示
        if let Some(rhs) = &self.rhs {
            rhs.visualize(&format!("{}{}", prefix, next_prefix), true);
        }
    }

    // 便利なラッパー関数
    #[allow(dead_code)]
    fn print_ast(&self) {
        println!("抽象構文木の表示:");
        self.visualize("", true);
    }
}

fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
    return equality(tokens);
}

fn equality(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = relational(tokens);
    loop {
        if consume(Symbol::Eq, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Eq),
                Some(node),
                Some(relational(tokens)),
            ));
        } else if consume(Symbol::Neq, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Neq),
                Some(node),
                Some(relational(tokens)),
            ));
        } else {
            return node;
        }
    }
}

fn relational(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = add(tokens);
    loop {
        if consume(Symbol::Lt, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Lt),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Le, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Le),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Gt, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Gt),
                Some(node),
                Some(add(tokens)),
            ));
        } else if consume(Symbol::Ge, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Ge),
                Some(node),
                Some(add(tokens)),
            ));
        } else {
            return node;
        }
    }
}

fn add(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(tokens);
    loop {
        if consume(Symbol::Add, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Add),
                Some(node),
                Some(mul(tokens)),
            ));
        } else if consume(Symbol::Sub, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Sub),
                Some(node),
                Some(mul(tokens)),
            ));
        } else {
            return node;
        }
    }
}

fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = unary(tokens);

    loop {
        if consume(Symbol::Mul, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Mul),
                Some(node),
                Some(unary(tokens)),
            ));
        } else if consume(Symbol::Div, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Div),
                Some(node),
                Some(unary(tokens)),
            ));
        } else {
            return node;
        }
    }
}

fn unary(tokens: &mut Vec<Token>) -> Box<Node> {
    if consume(Symbol::Add, tokens) {
        return primary(tokens);
    }
    if consume(Symbol::Sub, tokens) {
        return Box::new(Node::new(
            Token::Symbol(Symbol::Sub),
            Some(Box::new(Node::new(Token::Number(0), None, None))),
            Some(primary(tokens)),
        ));
    }
    return primary(tokens);
}

fn primary(tokens: &mut Vec<Token>) -> Box<Node> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Symbol::ParenL, tokens) {
        let node = expr(tokens);
        let _ = consume(Symbol::ParenR, tokens);
        return node;
    }
    // そうでなければ数値のはず
    return Box::new(Node::new(expect_number(tokens), None, None));
}

fn consume(op: Symbol, tokens: &mut Vec<Token>) -> bool {
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

fn expect_number(tokens: &mut Vec<Token>) -> Token {
    if tokens.is_empty() {
        eprintln!("error_1");
    }

    let next = tokens.first().unwrap();

    if !matches!(next, Token::Number(_)) {
        eprintln!("error_2");
    }

    let tmp = next.clone();
    tokens.remove(0);
    return tmp;
}

fn tokenize(input: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let symbol_potential: Vec<_> = Symbol::SYMBOLS.join("").chars().collect();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        if c.is_ascii_digit() {
            let mut number = String::new();
            while let Some(&digit) = chars.peek() {
                if digit.is_ascii_digit() {
                    number.push(digit);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(number.parse().unwrap()));
            continue;
        }

        if symbol_potential.contains(&c) {
            let mut symbol = String::new();
            while let Some(&sy) = chars.peek() {
                if symbol_potential.contains(&sy) {
                    symbol.push(sy);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Symbol(Symbol::classify(&symbol).unwrap()));
            continue;
        }
    }

    return tokens;
}

fn generate(node: Box<Node>, id_counter: &mut usize) -> String {
    match node.token {
        Token::Number(n) => {
            let name = format!("%tmp{}", *id_counter);
            println!("  {} = add i32 0, {}", name, n);
            *id_counter += 1;
            return name;
        }
        Token::Symbol(sym) => {
            let lhs = generate(node.lhs.unwrap(), id_counter);
            let rhs = generate(node.rhs.unwrap(), id_counter);
            let name = format!("%tmp{}", *id_counter);
            *id_counter += 1;

            let op = match sym {
                Symbol::Add => "add",
                Symbol::Sub => "sub",
                Symbol::Mul => "mul",
                Symbol::Div => "sdiv",
                _ => panic!("invalid operator"),
            };

            println!("  {} = {} i32 {}, {}", name, op, lhs, rhs);
            return name;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }
    println!("; ModuleID = 'main'");
    println!("define i32 @main() {{");

    let mut b = tokenize(args[1].to_string());
    let ast = expr(&mut b);

    let mut id_counter: usize = 0;
    generate(ast, &mut id_counter);
    println!("  ret i32 %tmp{}", id_counter - 1);
    println!("}}")
}

#[test]
fn test() {
    let a = " 5 !=  20 ";

    let mut b = tokenize(a.to_string());
    println!("トークン: {:?}", b);
    let ast = expr(&mut b);
    println!("{:?}", ast);
    ast.print_ast();
}
