#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    EqualEqual,   // '=='
    NotEqual,     // '!='
    Less,         // '<'
    LessEqual,    // '<='
    Greater,      // '>'
    GreaterEqual, // '>='
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Logical {
    AmpersandAmpersand, // '&&'
    PipePipe,           // '||'
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
    Logical(Logical),
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOp {
    Minus,      // -x
    Bang,       // !x
    Tilde,      // ~x
    Ampersand,  // &x
    Asterisk,   // *x
    PlusPlus,   // ++x
    MinusMinus, // --x
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PostfixOp {
    PlusPlus,   // x++
    MinusMinus, // x--
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
