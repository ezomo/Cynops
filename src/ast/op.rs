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
    Equal,               // '='
    PlusEqual,           // '+='
    MinusEqual,          // '-='
    AsteriskEqual,       // '*='
    SlashEqual,          // '/='
    PercentEqual,        // '%='
    CaretEqual,          // '^='
    PipeEqual,           // '|='
    LessLessEqual,       // '<<='
    GreaterGreaterEqual, // '>>='
    AmpersandEqual,      // '&='
}
impl AssignOp {
    pub fn equal() -> Self {
        AssignOp::Equal
    }

    pub fn plus_equal() -> Self {
        AssignOp::PlusEqual
    }

    pub fn minus_equal() -> Self {
        AssignOp::MinusEqual
    }

    pub fn asterisk_equal() -> Self {
        AssignOp::AsteriskEqual
    }

    pub fn slash_equal() -> Self {
        AssignOp::SlashEqual
    }

    pub fn percent_equal() -> Self {
        AssignOp::PercentEqual
    }

    pub fn caret_equal() -> Self {
        AssignOp::CaretEqual
    }

    pub fn pipe_equal() -> Self {
        AssignOp::PipeEqual
    }

    pub fn less_less_equal() -> Self {
        AssignOp::LessLessEqual
    }

    pub fn greater_greater_equal() -> Self {
        AssignOp::GreaterGreaterEqual
    }

    pub fn ampersand_equal() -> Self {
        AssignOp::AmpersandEqual
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum UnaryOp {
    Minus,      // -x
    Bang,       // !x
    Tilde,      // ~x
    Ampersand,  // &x
    Asterisk,   // *x
    PlusPlus,   // ++x
    MinusMinus, // --x
}
impl UnaryOp {
    pub fn minus() -> Self {
        UnaryOp::Minus // -x
    }

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

    pub fn plus_plus() -> Self {
        UnaryOp::PlusPlus // ++x
    }
    pub fn minus_minus() -> Self {
        UnaryOp::MinusMinus // --x
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum PostfixOp {
    PlusPlus,   // x++
    MinusMinus, // x--
}
impl PostfixOp {
    pub fn plus_plus() -> Self {
        PostfixOp::PlusPlus // x++
    }

    pub fn minus_minus() -> Self {
        PostfixOp::MinusMinus // x--
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
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
