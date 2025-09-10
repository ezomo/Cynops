use super::{DeclStmt, Ident, TypedExpr};
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Block {
    pub statements: Vec<Box<Stmt>>,
}
impl Block {
    pub fn new(statements: Vec<Box<Stmt>>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct If {
    pub cond: Box<TypedExpr>,   // 条件は式
    pub then_branch: Box<Stmt>, // ブロックや文
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Case {
    pub const_expr: TypedExpr,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct DefaultCase {
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum SwitchCase {
    Case(Case),
    Default(DefaultCase),
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Switch {
    pub cond: Box<TypedExpr>,
    pub cases: Vec<SwitchCase>,
}
impl SwitchCase {
    pub fn case(expr: TypedExpr, stmts: Vec<Box<Stmt>>) -> Self {
        SwitchCase::Case(Case {
            const_expr: expr,
            stmts,
        })
    }

    pub fn default(stmts: Vec<Box<Stmt>>) -> Self {
        SwitchCase::Default(DefaultCase { stmts })
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct While {
    pub cond: Box<TypedExpr>,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct DoWhile {
    pub body: Box<Stmt>,
    pub cond: Box<TypedExpr>, // 条件は式
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct For {
    pub init: Option<Box<TypedExpr>>, // 式（文じゃない）← int i = 0; はNG
    pub cond: Option<Box<TypedExpr>>, // 式
    pub step: Option<Box<TypedExpr>>, // 式（例: y += 1, x--）
    pub body: Box<Stmt>,              // 本体（文）
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Return {
    pub value: Option<Box<TypedExpr>>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Label {
    pub name: Ident,
    pub stmt: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Goto {
    pub label: Ident,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Control {
    If(If),
    While(While),
    DoWhile(DoWhile),
    For(For),
    Switch(Switch),
}

impl Control {
    pub fn r#if(cond: TypedExpr, then_branch: Stmt, else_branch: Option<Stmt>) -> Self {
        Self::If(If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    pub fn r#switch(cond: TypedExpr, cases: Vec<SwitchCase>) -> Self {
        Self::Switch(Switch {
            cond: Box::new(cond),
            cases,
        })
    }

    pub fn r#while(cond: TypedExpr, body: Stmt) -> Self {
        Self::While(While {
            cond: Box::new(cond),
            body: Box::new(body),
        })
    }
    pub fn r#do_while(body: Stmt, cond: TypedExpr) -> Self {
        Self::DoWhile(DoWhile {
            body: Box::new(body),
            cond: Box::new(cond),
        })
    }

    pub fn r#for(
        init: Option<TypedExpr>,
        cond: Option<TypedExpr>,
        step: Option<TypedExpr>,
        body: Stmt,
    ) -> Self {
        Self::For(For {
            init: init.map(|e| Box::new(e)),
            cond: cond.map(|e| Box::new(e)),
            step: step.map(|e| Box::new(e)),
            body: Box::new(body),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Stmt {
    ExprStmt(TypedExpr), // 式文（関数呼び出し、代入など）
    DeclStmt(DeclStmt),
    Control(Control),
    Return(Return),
    Goto(Goto),
    Label(Label),
    Block(Block),
    Break,
    Continue,
}
impl Stmt {
    pub fn expr(expr: TypedExpr) -> Self {
        Stmt::ExprStmt(expr)
    }

    pub fn decl_stmt(decl_stmt: DeclStmt) -> Self {
        Stmt::DeclStmt(decl_stmt)
    }

    pub fn control(control: Control) -> Self {
        Stmt::Control(control)
    }

    pub fn r#return(value: Option<TypedExpr>) -> Self {
        Stmt::Return(Return {
            value: value.map(|v| Box::new(v)),
        })
    }

    pub fn goto(label: Ident) -> Self {
        Stmt::Goto(Goto { label })
    }

    pub fn label(name: Ident, stmt: Stmt) -> Self {
        Stmt::Label(Label {
            name,
            stmt: Box::new(stmt),
        })
    }

    pub fn block(block: Block) -> Self {
        Stmt::Block(block)
    }

    pub fn r#break() -> Self {
        Stmt::Break
    }

    pub fn r#continue() -> Self {
        Stmt::Continue
    }
}
