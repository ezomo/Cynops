use super::{AssignOp, BinaryOp, Ident, PostfixOp, UnaryOp};

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
    Subscript(Subscript), // 添え字アクセス（配列やポインタ）
    Assign(Assign),
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
pub struct Call {
    pub func: Box<Expr>,
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Subscript {
    pub name: Box<Expr>,
    pub index: Box<Expr>,
}

impl Expr {
    pub fn num(n: usize) -> Self {
        Expr::Num(n)
    }

    pub fn char_lit(c: char) -> Self {
        Expr::Char(c)
    }

    pub fn ident(name: Ident) -> Self {
        Expr::Ident(name)
    }

    pub fn unary(op: UnaryOp, expr: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Unary(Unary { op, expr }))
    }

    pub fn postfix(op: PostfixOp, expr: Expr) -> Self {
        Expr::Postfix(Postfix {
            op,
            expr: Box::new(expr),
        })
    }

    pub fn binary(op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Binary(Binary { op, lhs, rhs }))
    }

    pub fn ternary(cond: Box<Expr>, then_branch: Expr, else_branch: Expr) -> Box<Self> {
        Box::new(Expr::Ternary(Ternary {
            cond,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }))
    }

    pub fn assign(op: AssignOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Self> {
        Box::new(Expr::Assign(Assign { op, lhs, rhs }))
    }

    pub fn call(func: Expr, args: Vec<Box<Expr>>) -> Self {
        Expr::Call(Call {
            func: Box::new(func),
            args,
        })
    }
    pub fn subscript(name: Expr, index: Expr) -> Self {
        Expr::Subscript(Subscript {
            name: Box::new(name),
            index: Box::new(index),
        })
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
