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
pub enum ExprSymbol {
    Arithmetic(Arithmetic),
    Parentheses(Parentheses),
    Comparison(Comparison),
    Assignment,
    Stop,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(usize),
    Ident(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ControlStructure {
    If,
    Else,
    For,
    While,
    Return,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ControlStructure(ControlStructure),
    ExprSymbol(ExprSymbol),
    Value(Value),
}

impl Token {
    pub const fn arith(a: Arithmetic) -> Self {
        Self::ExprSymbol(ExprSymbol::Arithmetic(a))
    }
    pub const fn comp(c: Comparison) -> Self {
        Self::ExprSymbol(ExprSymbol::Comparison(c))
    }
    pub const fn paren(p: Parentheses) -> Self {
        Self::ExprSymbol(ExprSymbol::Parentheses(p))
    }
    pub const fn assign() -> Self {
        Self::ExprSymbol(ExprSymbol::Assignment)
    }
    pub const fn stop() -> Self {
        Self::ExprSymbol(ExprSymbol::Stop)
    }
    pub const fn ctrl(c: ControlStructure) -> Self {
        Self::ControlStructure(c)
    }
    pub const fn number(n: usize) -> Self {
        Self::Value(Value::Number(n))
    }
    pub fn ident(name: impl Into<String>) -> Self {
        Self::Value(Value::Ident(name.into()))
    }
}

impl Token {
    pub const SYMBOLS: [(&str, Self); 19] = [
        ("+", Self::arith(Arithmetic::Add)),
        ("-", Self::arith(Arithmetic::Sub)),
        ("*", Self::arith(Arithmetic::Mul)),
        ("/", Self::arith(Arithmetic::Div)),
        ("(", Self::paren(Parentheses::L)),
        (")", Self::paren(Parentheses::R)),
        ("==", Self::comp(Comparison::Eq)),
        ("!=", Self::comp(Comparison::Neq)),
        ("<", Self::comp(Comparison::Lt)),
        ("<=", Self::comp(Comparison::Le)),
        (">", Self::comp(Comparison::Gt)),
        (">=", Self::comp(Comparison::Ge)),
        ("=", Self::assign()),
        (";", Self::stop()),
        ("if", Self::ctrl(ControlStructure::If)),
        ("else", Self::ctrl(ControlStructure::Else)),
        ("while", Self::ctrl(ControlStructure::For)),
        ("for", Self::ctrl(ControlStructure::While)),
        ("return", Self::ctrl(ControlStructure::Return)),
    ];

    pub fn classify(input: &str) -> Option<Self> {
        for (symbol, token) in Self::SYMBOLS.iter() {
            if *symbol == input {
                return Some(token.clone());
            }
        }
        None
    }
}

// 抽象構文木のノードの型
#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Expr {
        op: ExprSymbol,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Value(Value),
    If {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    Return {
        value: Box<Node>,
    },
}
