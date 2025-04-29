use std::collections::HashMap;

pub mod token {
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

    #[derive(Debug, PartialEq, Clone, Copy)]
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
}

pub mod node {
    use crate::setting::token::{ExprSymbol, Value};
    #[derive(Debug, PartialEq, Clone)]
    pub struct Expr {
        pub op: ExprSymbol,
        pub lhs: Box<Node>,
        pub rhs: Box<Node>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct If {
        pub condition: Box<Node>,
        pub then_branch: Box<Node>,
        pub else_branch: Option<Box<Node>>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Return {
        pub value: Box<Node>,
    }
    #[derive(Debug, PartialEq, Clone)]
    pub enum Control {
        If(If),
        Return(Return),
    }
    // 抽象構文木のノードの型
    #[derive(Debug, PartialEq, Clone)]
    pub enum Node {
        Value(Value),
        Expr(Expr),
        Control(Control),
    }
    impl Node {
        pub fn value(val: Value) -> Box<Self> {
            Box::new(Node::Value(val))
        }

        pub fn expr(op: ExprSymbol, lhs: Box<Node>, rhs: Box<Node>) -> Box<Self> {
            Box::new(Node::Expr(Expr { op, lhs, rhs }))
        }

        pub fn r#return(val: Box<Node>) -> Box<Self> {
            Box::new(Node::Control(Control::Return(Return { value: val })))
        }

        pub fn r#if(
            cond: Box<Node>,
            then_branch: Box<Node>,
            else_branch: Option<Box<Node>>,
        ) -> Box<Self> {
            Box::new(Node::Control(Control::If(If {
                condition: cond,
                then_branch,
                else_branch,
            })))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TmpNameGen {
    counter: usize,
}

impl TmpNameGen {
    pub fn new() -> Self {
        TmpNameGen { counter: 0 }
    }

    pub fn next(&mut self) -> String {
        let name = format!("%tmp{}", self.counter);
        self.counter += 1;
        name
    }
}

#[derive(Debug, PartialEq)]
pub struct CodeGenStatus {
    pub name_gen: TmpNameGen,
    pub variables: HashMap<String, String>,
}

impl CodeGenStatus {
    pub fn new() -> Self {
        Self {
            name_gen: TmpNameGen::new(),
            variables: HashMap::new(),
        }
    }
}
