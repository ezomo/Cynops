#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String), // identifier (e.g. foo, var1, x)
    Num(usize),    // integer literal (e.g. 123)
    Char(char),    // character literal (e.g. 'a', 'Z')

    Plus,     // '+'
    Minus,    // '-'
    Asterisk, // '*'
    Slash,    // '/'

    Equal,      // '='
    EqualEqual, // '=='
    NotEqual,   // '!='

    Less,         // '<'
    LessEqual,    // '<='
    Greater,      // '>'
    GreaterEqual, // '>='

    LParen,    // '('
    RParen,    // ')'
    LBrace,    // '{'
    RBrace,    // '}'
    Semicolon, // ';'
    Comma,     // ','

    Keyword(Keyword), // keywords (int, return, if, etc.)

    Eof, // end of input
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Int,    // 'int'
    Char,   // 'char'
    Return, // 'return'
    If,     // 'if'
    Else,   // 'else'
    While,  // 'while'
    For,    // 'for'
    Void,   // 'void'
}

impl Token {
    pub const SYMBOLS: [(&str, Self); 22] = [
        ("+", Self::Plus),
        ("-", Self::Minus),
        ("*", Self::Asterisk),
        ("/", Self::Slash),
        ("(", Self::LParen),
        (")", Self::RParen),
        ("{", Self::LBrace),
        ("}", Self::RBrace),
        ("=", Self::Equal),
        ("==", Self::EqualEqual),
        ("!=", Self::NotEqual),
        ("<", Self::Less),
        ("<=", Self::LessEqual),
        (">", Self::Greater),
        (">=", Self::GreaterEqual),
        (";", Self::Semicolon),
        (",", Self::Comma),
        ("if", Self::Keyword(Keyword::If)),
        ("else", Self::Keyword(Keyword::Else)),
        ("while", Self::Keyword(Keyword::While)),
        ("for", Self::Keyword(Keyword::For)),
        ("return", Self::Keyword(Keyword::Return)),
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
