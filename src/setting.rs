#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Arithmetic {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Parentheses {
    L, // (
    R, // )
}

#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Comparison {
    Eq,  // ==
    Neq, // !=
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
}

#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Symbol {
    Arithmetic(Arithmetic),
    Parentheses(Parentheses),
    Comparison(Comparison),
    Assignment,
    Stop,
}
impl Symbol {
    pub const SYMBOLS: [&str; 14] = [
        "+", "-", "*", "/", "(", ")", "==", "!=", "<", "<=", ">", ">=", "=", ";",
    ];
    pub fn classify(input: &str) -> Option<Self> {
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
            "=" => Some(Self::Assignment),
            ";" => Some(Self::Stop),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Token {
    Number(usize),  // 数値リテラル
    Symbol(Symbol), // 記号トークン
    Ident(char),
}

// 抽象構文木のノードの型
#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub token: Token,           // ノードの型
    pub lhs: Option<Box<Node>>, // 左辺
    pub rhs: Option<Box<Node>>, // 右辺
}
impl Node {
    pub fn new(token: Token, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
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
    pub fn visualize(&self, prefix: &str, is_last: bool) {
        let display_prefix = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last { "    " } else { "│   " };

        // ノードの値を表示
        let node_value = match &self.token {
            Token::Number(n) => format!("{}", n),
            Token::Ident(n) => format!("{}", n),
            Token::Symbol(s) => match s {
                Symbol::Arithmetic(Arithmetic::Add) => "+".to_string(),
                Symbol::Arithmetic(Arithmetic::Sub) => "-".to_string(),
                Symbol::Arithmetic(Arithmetic::Mul) => "*".to_string(),
                Symbol::Arithmetic(Arithmetic::Div) => "/".to_string(),
                Symbol::Parentheses(Parentheses::L) => "(".to_string(),
                Symbol::Parentheses(Parentheses::R) => ")".to_string(),
                Symbol::Comparison(Comparison::Eq) => "==".to_string(),
                Symbol::Comparison(Comparison::Neq) => "!=".to_string(),
                Symbol::Comparison(Comparison::Lt) => "<)".to_string(),
                Symbol::Comparison(Comparison::Le) => "<=".to_string(),
                Symbol::Comparison(Comparison::Gt) => ">)".to_string(),
                Symbol::Comparison(Comparison::Ge) => ">=".to_string(),
                Symbol::Assignment => "=".to_string(),
                _ => todo!(),
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
    pub fn print_ast(&self) {
        println!("抽象構文木の表示:");
        self.visualize("", true);
    }
}
