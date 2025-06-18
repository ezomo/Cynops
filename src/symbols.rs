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
pub struct Ternary {
    pub cond: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub op: AssignOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
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
    Ternary(Ternary), // 三項演算子（条件 ? 真の値 : 偽の値）
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
pub struct Case {
    pub expr: Expr,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DefaultCase {
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SwitchCase {
    Case(Case),
    Default(DefaultCase),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Switch {
    pub cond: Box<Expr>,
    pub cases: Vec<SwitchCase>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub cond: Box<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DoWhile {
    pub body: Box<Stmt>,
    pub cond: Box<Expr>, // 条件は式
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
pub struct Label {
    pub name: Ident,
    pub stmt: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Goto {
    pub label: Ident,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Control {
    If(If),
    While(While),
    DoWhile(DoWhile),
    For(For),
    Switch(Switch),
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
pub enum ParamList {
    Void,               // f(void)
    Params(Vec<Param>), // f(int x, char y)
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSig {
    pub ret_type: Type,
    pub name: Ident,
    pub params: ParamList,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionProto {
    pub sig: FunctionSig,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDef {
    pub sig: FunctionSig,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    FunctionDef(FunctionDef),
    FunctionProto(FunctionProto),
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
    Goto(Goto),
    Label(Label),
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

    pub fn function_proto(proto: FunctionProto) -> Self {
        TopLevel::FunctionProto(proto)
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

    pub fn r#switch(cond: Expr, cases: Vec<SwitchCase>) -> Box<Self> {
        Box::new(Stmt::Control(Control::Switch(Switch {
            cond: Box::new(cond),
            cases,
        })))
    }

    pub fn r#while(cond: Expr, body: Stmt) -> Box<Self> {
        Box::new(Stmt::Control(Control::While(While {
            cond: Box::new(cond),
            body: Box::new(body),
        })))
    }
    pub fn r#do_while(body: Stmt, cond: Expr) -> Box<Self> {
        Box::new(Stmt::Control(Control::DoWhile(DoWhile {
            body: Box::new(body),
            cond: Box::new(cond),
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

    pub fn goto(label: Ident) -> Box<Self> {
        Box::new(Stmt::Goto(Goto { label }))
    }

    pub fn label(name: Ident, stmt: Stmt) -> Box<Self> {
        Box::new(Stmt::Label(Label {
            name,
            stmt: Box::new(stmt),
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

    pub fn ternary(cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Ternary(Ternary {
            cond,
            then_branch,
            else_branch,
        }))
    }

    pub fn assign(op: AssignOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Assign(Assign { op, lhs, rhs }))
    }

    pub fn call(func: Ident, args: Vec<Box<Expr>>) -> Box<Self> {
        Box::new(Expr::Call(Call { func: func, args }))
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

impl FunctionSig {
    pub fn new(ret_type: Type, name: Ident, params: ParamList) -> Self {
        Self {
            ret_type,
            name,
            params,
        }
    }
}

impl FunctionProto {
    pub fn new(sig: FunctionSig) -> Box<Self> {
        Box::new(Self { sig })
    }
}

impl FunctionDef {
    pub fn new(sig: FunctionSig, body: Block) -> Box<Self> {
        Box::new(Self { sig, body })
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
    pub fn plus_plus() -> Self {
        PostfixOp::PlusPlus // x++
    }

    pub fn minus_minus() -> Self {
        PostfixOp::MinusMinus // x--
    }
}

impl SwitchCase {
    pub fn case(expr: Expr, stmts: Vec<Box<Stmt>>) -> Self {
        SwitchCase::Case(Case { expr, stmts })
    }

    pub fn default(stmts: Vec<Box<Stmt>>) -> Self {
        SwitchCase::Default(DefaultCase { stmts })
    }
}
