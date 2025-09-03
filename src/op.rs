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

impl ToString for Arithmetic {
    fn to_string(&self) -> String {
        match self {
            Arithmetic::Plus => "+".to_string(),
            Arithmetic::Minus => "-".to_string(),
            Arithmetic::Asterisk => "*".to_string(),
            Arithmetic::Slash => "/".to_string(),
            Arithmetic::Percent => "%".to_string(),
            Arithmetic::Caret => "^".to_string(),
            Arithmetic::Pipe => "|".to_string(),
            Arithmetic::LessLess => "<<".to_string(),
            Arithmetic::GreaterGreater => ">>".to_string(),
            Arithmetic::Ampersand => "&".to_string(),
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

impl ToString for Comparison {
    fn to_string(&self) -> String {
        match self {
            Comparison::EqualEqual => "==".to_string(),
            Comparison::NotEqual => "!=".to_string(),
            Comparison::Less => "<".to_string(),
            Comparison::LessEqual => "<=".to_string(),
            Comparison::Greater => ">".to_string(),
            Comparison::GreaterEqual => ">=".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Logical {
    AmpersandAmpersand, // '&&'
    PipePipe,           // '||'
}

impl ToString for Logical {
    fn to_string(&self) -> String {
        match self {
            Logical::AmpersandAmpersand => "&&".to_string(),
            Logical::PipePipe => "||".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
    Logical(Logical),
}

impl ToString for BinaryOp {
    fn to_string(&self) -> String {
        match self {
            BinaryOp::Arithmetic(op) => op.to_string(),
            BinaryOp::Comparison(op) => op.to_string(),
            BinaryOp::Logical(op) => op.to_string(),
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

impl ToString for AssignOp {
    fn to_string(&self) -> String {
        match self {
            AssignOp::Equal => "=".to_string(),
            AssignOp::PlusEqual => "+=".to_string(),
            AssignOp::MinusEqual => "-=".to_string(),
            AssignOp::AsteriskEqual => "*=".to_string(),
            AssignOp::SlashEqual => "/=".to_string(),
            AssignOp::PercentEqual => "%=".to_string(),
            AssignOp::CaretEqual => "^=".to_string(),
            AssignOp::PipeEqual => "|=".to_string(),
            AssignOp::LessLessEqual => "<<=".to_string(),
            AssignOp::GreaterGreaterEqual => ">>=".to_string(),
            AssignOp::AmpersandEqual => "&=".to_string(),
        }
    }
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

impl ToString for UnaryOp {
    fn to_string(&self) -> String {
        match self {
            UnaryOp::Minus => "-".to_string(),
            UnaryOp::Bang => "!".to_string(),
            UnaryOp::Tilde => "~".to_string(),
            UnaryOp::Ampersand => "&".to_string(),
            UnaryOp::Asterisk => "*".to_string(),
            UnaryOp::PlusPlus => "++".to_string(),
            UnaryOp::MinusMinus => "--".to_string(),
        }
    }
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

impl ToString for PostfixOp {
    fn to_string(&self) -> String {
        match self {
            PostfixOp::PlusPlus => "++".to_string(),
            PostfixOp::MinusMinus => "--".to_string(),
        }
    }
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

impl ToString for MemberAccessOp {
    fn to_string(&self) -> String {
        match self {
            MemberAccessOp::Dot => ".".to_string(),
            MemberAccessOp::MinusGreater => "->".to_string(),
        }
    }
}

impl MemberAccessOp {
    pub fn minus_greater() -> Self {
        Self::MinusGreater // ->
    }

    pub fn dot() -> Self {
        Self::Dot // .
    }
}
