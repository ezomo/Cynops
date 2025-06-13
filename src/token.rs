#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Num(usize),
    Char(char),
    Plus,             // +
    Minus,            // -
    Asterisk,         // *
    Slash,            // /
    Equal,            // =
    LParen,           // (
    RParen,           // )
    LBrace,           // {
    RBrace,           // }
    Semicolon,        // ;
    Comma,            // ,
    Keyword(Keyword), // int, return, if, etc.
    Eof,
}

// 予約語
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Int,
    Char,
    Return,
    If,
    Else,
    While,
    For,
    Void,
}
