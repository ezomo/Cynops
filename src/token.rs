use ordered_float::OrderedFloat;
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),               // e.g., foo
    NumInt(usize),               // e.g., 123
    NumFloat(OrderedFloat<f64>), // e.g., 123
    Char(char),                  // e.g., 'a'
    String(String),

    //other symbols
    Question, // '?'
    Colon,    // ':'

    // Arithmetic operators
    Plus,       // '+'
    Minus,      // '-'
    Asterisk,   // '*'
    Percent,    // '%'
    Slash,      // '/'
    PlusPlus,   // '++'
    MinusMinus, // '--'

    // Bitwise Operators
    Caret,          // '^'
    Pipe,           // '|'
    LessLess,       // '<<'
    GreaterGreater, // '>>'

    //logical operators
    AmpersandAmpersand, // '&&'
    PipePipe,           // '||'

    // Unary-specific operators
    Bang,      // '!'
    Tilde,     // '~'
    Ampersand, // '&'

    // Assignment
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

    // Comparison
    EqualEqual,   // '=='
    NotEqual,     // '!='
    Less,         // '<'
    LessEqual,    // '<='
    Greater,      // '>'
    GreaterEqual, // '>='

    // Delimiters
    LParen,    // '('
    RParen,    // ')'
    LBrace,    // '{'
    RBrace,    // '}'
    LBracket,  // '['
    RBracket,  // ']'
    Semicolon, // ';'
    Comma,     // ','

    //struct
    MinusGreater, //->
    Dot,          //.

    // Keywords
    Keyword(Keyword), // e.g., int, return
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Int,      // 'int'
    Char,     // 'char'
    Double,   //'double'
    Return,   // 'return'
    If,       // 'if'
    Else,     // 'else'
    While,    // 'while'
    Do,       // 'do'
    For,      // 'for'
    Void,     // 'void'
    Break,    // 'break'
    Continue, // 'continue'
    Switch,   // 'switch'
    Case,     // 'case'
    Default,  // 'default'
    Goto,     // 'goto'
    Struct,   // 'struct'
    Union,    // 'union'
    Enum,     // 'enum'
    Typedef,  // 'typedef'
    Sizeof,   // 'sizeof'
}

impl Token {
    pub const SYMBOLS: [(&str, Self); 66] = [
        ("?", Self::Question),
        (":", Self::Colon),
        ("+", Self::Plus),
        ("-", Self::Minus),
        ("*", Self::Asterisk),
        ("/", Self::Slash),
        ("%", Self::Percent),
        ("++", Self::PlusPlus),
        ("--", Self::MinusMinus),
        ("^", Self::Caret),
        ("|", Self::Pipe),
        ("<<", Self::LessLess),
        (">>", Self::GreaterGreater),
        ("&&", Self::AmpersandAmpersand),
        ("||", Self::PipePipe),
        ("!", Self::Bang),
        ("~", Self::Tilde),
        ("&", Self::Ampersand),
        ("(", Self::LParen),
        (")", Self::RParen),
        ("{", Self::LBrace),
        ("}", Self::RBrace),
        ("[", Self::LBracket),
        ("]", Self::RBracket),
        ("=", Self::Equal),
        ("+=", Self::PlusEqual),
        ("-=", Self::MinusEqual),
        ("*=", Self::AsteriskEqual),
        ("/=", Self::SlashEqual),
        ("%=", Self::PercentEqual),
        ("^=", Self::CaretEqual),
        ("|=", Self::PipeEqual),
        ("<<=", Self::LessLessEqual),
        (">>=", Self::GreaterGreaterEqual),
        ("&=", Self::AmpersandEqual),
        ("==", Self::EqualEqual),
        ("!=", Self::NotEqual),
        ("<", Self::Less),
        ("<=", Self::LessEqual),
        (">", Self::Greater),
        (">=", Self::GreaterEqual),
        (";", Self::Semicolon),
        (",", Self::Comma),
        (".", Self::Dot),
        ("->", Self::MinusGreater),
        ("int", Self::Keyword(Keyword::Int)),
        ("double", Self::Keyword(Keyword::Double)),
        ("char", Self::Keyword(Keyword::Char)),
        ("void", Self::Keyword(Keyword::Void)),
        ("if", Self::Keyword(Keyword::If)),
        ("else", Self::Keyword(Keyword::Else)),
        ("while", Self::Keyword(Keyword::While)),
        ("do", Self::Keyword(Keyword::Do)),
        ("for", Self::Keyword(Keyword::For)),
        ("return", Self::Keyword(Keyword::Return)),
        ("break", Self::Keyword(Keyword::Break)),
        ("continue", Self::Keyword(Keyword::Continue)),
        ("switch", Self::Keyword(Keyword::Switch)),
        ("case", Self::Keyword(Keyword::Case)),
        ("default", Self::Keyword(Keyword::Default)),
        ("goto", Self::Keyword(Keyword::Goto)),
        ("struct", Self::Keyword(Keyword::Struct)),
        ("union", Self::Keyword(Keyword::Union)),
        ("enum", Self::Keyword(Keyword::Enum)),
        ("typedef", Self::Keyword(Keyword::Typedef)),
        ("sizeof", Self::Keyword(Keyword::Sizeof)),
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

impl Token {
    pub fn r#return() -> Self {
        Token::Keyword(Keyword::Return)
    }

    pub fn r#if() -> Self {
        Token::Keyword(Keyword::If)
    }

    pub fn r#else() -> Self {
        Token::Keyword(Keyword::Else)
    }

    pub fn r#while() -> Self {
        Token::Keyword(Keyword::While)
    }

    pub fn r#do() -> Self {
        Token::Keyword(Keyword::Do)
    }

    pub fn r#for() -> Self {
        Token::Keyword(Keyword::For)
    }
    pub fn r#break() -> Self {
        Token::Keyword(Keyword::Break)
    }
    pub fn r#continue() -> Self {
        Token::Keyword(Keyword::Continue)
    }
    pub fn r#switch() -> Self {
        Token::Keyword(Keyword::Switch)
    }
    pub fn r#case() -> Self {
        Token::Keyword(Keyword::Case)
    }
    pub fn r#default() -> Self {
        Token::Keyword(Keyword::Default)
    }
    pub fn r#goto() -> Self {
        Token::Keyword(Keyword::Goto)
    }
    pub fn r#struct() -> Self {
        Token::Keyword(Keyword::Struct)
    }

    pub fn r#union() -> Self {
        Token::Keyword(Keyword::Union)
    }

    pub fn r#enum() -> Self {
        Token::Keyword(Keyword::Enum)
    }

    pub fn r#typedef() -> Self {
        Token::Keyword(Keyword::Typedef)
    }

    pub fn r#sizeof() -> Self {
        Token::Keyword(Keyword::Sizeof)
    }
}
