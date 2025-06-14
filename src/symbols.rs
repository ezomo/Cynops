#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Arithmetic {
    Plus,     // '+'
    Minus,    // '-'
    Asterisk, // '*'
    Slash,    // '/'
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
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Num(usize),
    Char(char),
    Ident(String),
    Binary(Binary),
    Call(Call),
    Assign(Assign),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Void,
    Int,
    Char,
    Pointer(Box<Type>),
}

// ステートメント（文）
#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub cond: Box<Expr>,        // 条件は式
    pub then_branch: Box<Stmt>, // ブロックや文
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub cond: Box<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub init: Option<Box<Stmt>>,
    pub cond: Option<Box<Expr>>,
    pub step: Option<Box<Stmt>>,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Control {
    If(If),
    While(While),
    For(For),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Decl {
    pub ty: Type,
    pub name: String,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func: String,
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDef {
    pub ret_type: Type,
    pub name: String,
    pub params: Vec<(Type, String)>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    ExprStmt(Expr), // 式文（関数呼び出し、代入など）
    Decl(Decl),
    Control(Control),
    Return(Return),
    Block(Block),
    FunctionDef(FunctionDef),
}
