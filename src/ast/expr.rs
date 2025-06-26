use crate::ast::MemberAccessOp;

use super::{AssignOp, BinaryOp, Ident, PostfixOp, UnaryOp};

#[derive(Debug, PartialEq, Clone)]
pub struct Postfix {
    pub expr: Box<Expr>,
    pub op: PostfixOp,
}
#[derive(Debug, PartialEq, Clone)]
pub struct PostfixD {
    pub base: Expr,                    // primary に相当する基の式
    pub suffixes: Vec<PostfixDSuffix>, // 後置操作の連続
}

#[derive(Debug, PartialEq, Clone)]
pub enum PostfixDSuffix {
    ArrayAcsess(Expr),                   // [ expr ]
    ArgList(Vec<Box<Expr>>),             // ( arg_list )
    PostfixOp(PostfixOp),                // ++, --
    MemberAccess(MemberAccessOp, Ident), // . ident または -> ident
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

#[derive(Debug, PartialEq, Clone)]
pub struct MemberAccess {
    pub base: Box<Expr>,      // 左側 (構造体 or ポインタ)
    pub member: Ident,        // アクセスされるメンバ名
    pub kind: MemberAccessOp, // . or ->
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Char(char),
    Ident(Ident),
    Num(usize),
    Postfix(Postfix),
    Subscript(Subscript),
    MemberAccess(MemberAccess),
    Ternary(Ternary),
    Unary(Unary),
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

    pub fn member_access(base: Expr, member: Ident, kind: MemberAccessOp) -> Self {
        Expr::MemberAccess(MemberAccess {
            base: Box::new(base),
            member: member,
            kind,
        })
    }
}
