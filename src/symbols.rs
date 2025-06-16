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
pub enum BinaryOp {
    Arithmetic(Arithmetic),
    Comparison(Comparison),
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

#[derive(Debug, PartialEq, Clone)]
pub struct Postfix {
    pub op: PostfixOp,
    pub expr: Box<Expr>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
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
pub struct Ident {
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Num(usize),
    Char(char),
    Ident(Ident),
    Binary(Binary),
    Unary(Unary),
    Postfix(Postfix),
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
    pub init: Option<Box<Expr>>, // 式（文じゃない）← int i = 0; はNG
    pub cond: Option<Box<Expr>>, // 式
    pub step: Option<Box<Expr>>, // 式（例: y += 1, x--）
    pub body: Box<Stmt>,         // 本体（文）
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
    pub name: Ident,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func: Ident,
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: Ident,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDef {
    pub ret_type: Type,
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    FunctionDef(FunctionDef),
    Stmt(Stmt),
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
    Break,
    Continue,
}

impl Program {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl TopLevel {
    pub fn function_def(def: FunctionDef) -> Self {
        TopLevel::FunctionDef(def)
    }

    pub fn stmt(stmt: Stmt) -> Self {
        TopLevel::Stmt(stmt)
    }
}

impl Stmt {
    pub fn expr(expr: Expr) -> Box<Self> {
        Box::new(Stmt::ExprStmt(expr))
    }

    pub fn decl(ty: Type, name: Ident, init: Option<Expr>) -> Box<Self> {
        Box::new(Stmt::Decl(Decl {
            ty,
            name,
            init: init.map(Box::new),
        }))
    }

    pub fn r#if(cond: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Box<Self> {
        Box::new(Stmt::Control(Control::If(If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })))
    }

    pub fn r#while(cond: Expr, body: Stmt) -> Box<Self> {
        Box::new(Stmt::Control(Control::While(While {
            cond: Box::new(cond),
            body: Box::new(body),
        })))
    }

    pub fn r#for(
        init: Option<Expr>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Stmt,
    ) -> Box<Self> {
        Box::new(Stmt::Control(Control::For(For {
            init: init.map(Box::new),
            cond: cond.map(Box::new),
            step: step.map(Box::new),
            body: Box::new(body),
        })))
    }

    pub fn r#return(value: Option<Expr>) -> Box<Self> {
        Box::new(Stmt::Return(Return {
            value: value.map(Box::new),
        }))
    }

    pub fn block(block: Block) -> Box<Self> {
        Box::new(Stmt::Block(block))
    }
    pub fn r#break() -> Box<Self> {
        Box::new(Stmt::Break)
    }
    pub fn r#continue() -> Box<Self> {
        Box::new(Stmt::Continue)
    }
}

impl Expr {
    pub fn num(n: usize) -> Box<Self> {
        Box::new(Expr::Num(n))
    }

    pub fn char_lit(c: char) -> Box<Self> {
        Box::new(Expr::Char(c))
    }

    pub fn ident(name: Ident) -> Box<Self> {
        Box::new(Expr::Ident(name))
    }

    pub fn unary(op: UnaryOp, expr: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Unary(Unary { op, expr }))
    }

    pub fn postfix(op: PostfixOp, expr: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Postfix(Postfix { op, expr }))
    }

    pub fn binary(op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Binary(Binary { op, lhs, rhs }))
    }

    pub fn assign(lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Assign(Assign { lhs, rhs }))
    }

    pub fn call(func: Ident, args: Vec<Box<Expr>>) -> Box<Self> {
        Box::new(Expr::Call(Call { func: func, args }))
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
}

impl Ident {
    /// 新しく Ident を作る（&str, String 両方対応）
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Type {
    pub fn pointer(inner: Type) -> Self {
        Type::Pointer(Box::new(inner))
    }
}

impl Param {
    pub fn new(ty: Type, name: Ident) -> Self {
        Self { ty, name }
    }
}

impl FunctionDef {
    pub fn new(ret_type: Type, name: Ident, params: Vec<Param>, body: Block) -> Box<Self> {
        Box::new(Self {
            ret_type,
            name,
            params,
            body,
        })
    }
}

impl Block {
    pub fn new(statements: Vec<Box<Stmt>>) -> Box<Self> {
        Box::new(Self { statements })
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

impl PostfixOp {
    pub fn post_inc() -> Self {
        PostfixOp::PlusPlus // x++
    }

    pub fn post_dec() -> Self {
        PostfixOp::MinusMinus // x--
    }
}
