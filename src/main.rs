use std::char;
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
}
impl Symbol {
    fn classify(input: char) -> Option<Self> {
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

fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
    let mut node = primary(tokens);

    loop {
        if consume(Symbol::Mul, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Mul),
                Some(node),
                Some(primary(tokens)),
            ));
        } else if consume(Symbol::Div, tokens) {
            node = Box::new(Node::new(
                Token::Symbol(Symbol::Div),
                Some(node),
                Some(primary(tokens)),
            ));
        } else {
            return node;
        }
    }
}

fn expr(tokens: &mut Vec<Token>) -> Box<Node> {
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

fn tokenize(string: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
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
            tokens.push(Token::Number(stack.join("").parse().unwrap()));
            stack.clear();
        }
        let symbol = Symbol::classify(c);
        if symbol.is_some() {
            tokens.push(Token::Symbol(symbol.unwrap()));
        }
    }

    if !stack.is_empty() {
        tokens.push(Token::Number(stack.join("").parse().unwrap()));
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
