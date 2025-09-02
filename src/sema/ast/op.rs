use crate::ast;

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Arithmetic {
    Plus,           // '+'
    Minus,          // '-'
    Asterisk,       // '*'
    Slash,          // '/'
    Percent,        // '%'
    Caret,          // '^'
    Pipe,           // '|'
    LessLess,       // '<<'
    GreaterGreater, // '>>'
    Ampersand,      // '&'
}

impl ast::Arithmetic {
    fn as_same(&self) -> Arithmetic {
        match self {
            Self::Plus => Arithmetic::Plus,                     // '+'
            Self::Minus => Arithmetic::Minus,                   // '-'
            Self::Asterisk => Arithmetic::Asterisk,             // '*'
            Self::Slash => Arithmetic::Slash,                   // '/'
            Self::Percent => Arithmetic::Percent,               // '%'
            Self::Caret => Arithmetic::Caret,                   // '^'
            Self::Pipe => Arithmetic::Pipe,                     // '|'
            Self::LessLess => Arithmetic::LessLess,             // '<<'
            Self::GreaterGreater => Arithmetic::GreaterGreater, // '>>'
            Self::Ampersand => Arithmetic::Ampersand,           // '&'
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Comparison {
    EqualEqual,   // '=='
    NotEqual,     // '!='
    Less,         // '<'
    LessEqual,    // '<='
    Greater,      // '>'
    GreaterEqual, // '>='
}

impl ast::Comparison {
    pub fn as_same(&self) -> Comparison {
        match self {
            Self::EqualEqual => Comparison::EqualEqual,
            Self::NotEqual => Comparison::NotEqual,
            Self::Less => Comparison::Less,
            Self::LessEqual => Comparison::LessEqual,
            Self::Greater => Comparison::Greater,
            Self::GreaterEqual => Comparison::GreaterEqual,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Logical {
    AmpersandAmpersand, // '&&'
    PipePipe,           // '||'
}

impl ast::Logical {
    pub fn as_same(&self) -> Logical {
        match self {
            Self::AmpersandAmpersand => Logical::AmpersandAmpersand,
            Self::PipePipe => Logical::PipePipe,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
    Logical(Logical),
}

impl ast::BinaryOp {
    /// ASTのBinaryOpをcrate内共通型に変換
    pub fn as_same(&self) -> BinaryOp {
        match self {
            ast::BinaryOp::Arithmetic(op) => BinaryOp::Arithmetic(op.as_same()),
            ast::BinaryOp::Comparison(op) => BinaryOp::Comparison(op.as_same()),
            ast::BinaryOp::Logical(op) => BinaryOp::Logical(op.as_same()),
        }
    }
}

impl BinaryOp {
    pub fn plus() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Plus)
    }

    pub fn minus() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Minus)
    }

    pub fn asterisk() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Asterisk)
    }

    pub fn slash() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Slash)
    }
    pub fn percent() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Percent)
    }
    pub fn ampersand() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Ampersand)
    }
    pub fn pipe() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Pipe)
    }
    pub fn caret() -> Self {
        BinaryOp::Arithmetic(Arithmetic::Caret)
    }
    pub fn less_less() -> Self {
        BinaryOp::Arithmetic(Arithmetic::LessLess)
    }
    pub fn greater_greater() -> Self {
        BinaryOp::Arithmetic(Arithmetic::GreaterGreater)
    }

    pub fn equal_equal() -> Self {
        BinaryOp::Comparison(Comparison::EqualEqual)
    }

    pub fn not_equal() -> Self {
        BinaryOp::Comparison(Comparison::NotEqual)
    }

    pub fn less() -> Self {
        BinaryOp::Comparison(Comparison::Less)
    }

    pub fn less_equal() -> Self {
        BinaryOp::Comparison(Comparison::LessEqual)
    }

    pub fn greater() -> Self {
        BinaryOp::Comparison(Comparison::Greater)
    }

    pub fn greater_equal() -> Self {
        BinaryOp::Comparison(Comparison::GreaterEqual)
    }

    pub fn ampersand_ampersand() -> Self {
        BinaryOp::Logical(Logical::AmpersandAmpersand)
    }
    pub fn pipe_pipe() -> Self {
        BinaryOp::Logical(Logical::PipePipe)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum AssignOp {
    Equal, // '='
}

impl ast::AssignOp {
    /// ASTのAssignOpをcrate内共通型に変換
    pub fn as_same(&self) -> AssignOp {
        match self {
            Self::Equal => AssignOp::Equal,
            _ => unreachable!(),
        }
    }
}
impl AssignOp {
    pub fn equal() -> Self {
        AssignOp::Equal
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum UnaryOp {
    Bang,      // !x
    Tilde,     // ~x
    Ampersand, // &x
    Asterisk,  // *x
}
impl ast::UnaryOp {
    /// ASTのUnaryOpをcrate内共通型に変換
    pub fn as_same(&self) -> UnaryOp {
        match self {
            Self::Bang => UnaryOp::Bang,           // !
            Self::Tilde => UnaryOp::Tilde,         // ~
            Self::Ampersand => UnaryOp::Ampersand, // &
            Self::Asterisk => UnaryOp::Asterisk,   // *
            _ => unreachable!(),
        }
    }
}

impl UnaryOp {
    pub fn bang() -> Self {
        UnaryOp::Bang // !x
    }

    pub fn tilde() -> Self {
        UnaryOp::Tilde // ~x
    }

    pub fn ampersand() -> Self {
        UnaryOp::Ampersand // &x
    }

    pub fn asterisk() -> Self {
        UnaryOp::Asterisk // *x
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum MemberAccessOp {
    Dot,
    MinusGreater,
}

impl MemberAccessOp {
    pub fn minus_greater() -> Self {
        Self::MinusGreater // ->
    }

    pub fn dot() -> Self {
        Self::Dot // .
    }
}
