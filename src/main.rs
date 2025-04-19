use std::env;
use std::process;
use std::usize;

#[derive(Debug, PartialEq, Clone)]
enum Arithmetic {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
}

#[derive(Debug, PartialEq, Clone)]
enum Parentheses {
    L, // (
    R, // )
}

#[derive(Debug, PartialEq, Clone)]
enum Comparison {
    Eq,  // ==
    Neq, // !=
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
}

#[derive(Debug, PartialEq, Clone)]
enum Symbol {
    Arithmetic(Arithmetic),
    Parentheses(Parentheses),
    Comparison(Comparison),
}
impl Symbol {
    const SYMBOLS: [&str; 12] = [
        "+", "-", "*", "/", "(", ")", "==", "!=", "<", "<=", ">", ">=",
    ];
    fn classify(input: &str) -> Option<Self> {
        match input {
            "+" => Some(Self::Arithmetic(Arithmetic::Add)),
            "-" => Some(Self::Arithmetic(Arithmetic::Sub)),
            "*" => Some(Self::Arithmetic(Arithmetic::Mul)),
            "/" => Some(Self::Arithmetic(Arithmetic::Div)),
            "(" => Some(Self::Parentheses(Parentheses::L)),
            ")" => Some(Self::Parentheses(Parentheses::R)),
            "==" => Some(Self::Comparison(Comparison::Eq)),
            "!=" => Some(Self::Comparison(Comparison::Neq)),
            "<" => Some(Self::Comparison(Comparison::Lt)),
            "<=" => Some(Self::Comparison(Comparison::Le)),
            ">" => Some(Self::Comparison(Comparison::Gt)),
            ">=" => Some(Self::Comparison(Comparison::Ge)),
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
                Symbol::Arithmetic(Arithmetic::Add) => "Symbol(+)".to_string(),
                Symbol::Arithmetic(Arithmetic::Sub) => "Symbol(-)".to_string(),
                Symbol::Arithmetic(Arithmetic::Mul) => "Symbol(*)".to_string(),
                Symbol::Arithmetic(Arithmetic::Div) => "Symbol(/)".to_string(),
                Symbol::Parentheses(Parentheses::L) => "Symbol(()".to_string(),
                Symbol::Parentheses(Parentheses::R) => "Symbol())".to_string(),
                Symbol::Comparison(Comparison::Eq) => "Symbol(==)".to_string(),
                Symbol::Comparison(Comparison::Neq) => "Symbol(!=)".to_string(),
                Symbol::Comparison(Comparison::Lt) => "Symbol(<)".to_string(),
                Symbol::Comparison(Comparison::Le) => "Symbol(<=)".to_string(),
                Symbol::Comparison(Comparison::Gt) => "Symbol(>)".to_string(),
                Symbol::Comparison(Comparison::Ge) => "Symbol(>=)".to_string(),
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

fn relational(tokens: &mut Vec<Token>) -> Box<Node> {
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

fn add(tokens: &mut Vec<Token>) -> Box<Node> {
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

fn mul(tokens: &mut Vec<Token>) -> Box<Node> {
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

fn unary(tokens: &mut Vec<Token>) -> Box<Node> {
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

fn primary(tokens: &mut Vec<Token>) -> Box<Node> {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Symbol::Parentheses(Parentheses::L), tokens) {
        let node = expr(tokens);
        let _ = consume(Symbol::Parentheses(Parentheses::R), tokens);
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
                Symbol::Arithmetic(Arithmetic::Add) => "add".to_string(),
                Symbol::Arithmetic(Arithmetic::Sub) => "sub".to_string(),
                Symbol::Arithmetic(Arithmetic::Mul) => "mul".to_string(),
                Symbol::Arithmetic(Arithmetic::Div) => "sdiv".to_string(),
                Symbol::Comparison(Comparison::Eq) => "icmp eq".to_string(),
                Symbol::Comparison(Comparison::Neq) => "icmp ne".to_string(),
                Symbol::Comparison(Comparison::Lt) => "icmp slt".to_string(),
                Symbol::Comparison(Comparison::Le) => "icmp sle".to_string(),
                Symbol::Comparison(Comparison::Gt) => "icmp sgt".to_string(),
                Symbol::Comparison(Comparison::Ge) => "icmp sge".to_string(),
                _ => panic!("error"),
            };

            println!("  {} = {} i32 {}, {}", name, op, lhs, rhs);

            if matches!(sym, Symbol::Comparison(_)) {
                let name_1 = format!("%tmp{}", *id_counter);
                *id_counter += 1;
                println!("  {} = zext i1 {} to i32", name_1, name);
                return name_1;
            }
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
    let a = "1 !=1";

    let mut b = tokenize(a.to_string());
    println!("トークン: {:?}", b);
    let ast = expr(&mut b);
    println!("{:?}", ast);
    ast.print_ast();
}
