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

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Comparison {
    EqualEqual,   // '=='
    NotEqual,     // '!='
    Less,         // '<'
    LessEqual,    // '<='
    Greater,      // '>'
    GreaterEqual, // '>='
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Logical {
    AmpersandAmpersand, // '&&'
    PipePipe,           // '||'
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
    Logical(Logical),
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
