use super::{Ident, Symbol, Type};
use crate::op::*;
use ordered_float::OrderedFloat;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: Box<TypedExpr>,
    pub rhs: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Ternary {
    pub cond: Box<TypedExpr>,
    pub then_branch: Box<TypedExpr>,
    pub else_branch: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]

pub struct Assign {
    pub op: AssignOp,
    pub lhs: Box<TypedExpr>,
    pub rhs: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Call {
    pub func: Box<TypedExpr>,
    pub args: Vec<TypedExpr>,
}
impl Call {
    pub fn new(func: TypedExpr, args: Vec<TypedExpr>) -> Self {
        Call {
            func: Box::new(func),
            args: args,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Subscript {
    pub subject: Box<TypedExpr>,
    pub index: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct MemberAccess {
    pub base: Box<TypedExpr>, // 左側 (構造体 or ポインタ)
    pub member: Ident,        // アクセスされるメンバ名
    pub kind: MemberAccessOp, // . or ->
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Sizeof {
    Type(Type),
    TypedExpr(Box<TypedExpr>),
}

impl Sizeof {
    pub fn r#type(ty: Type) -> Self {
        Self::Type(ty)
    }
    pub fn r#expr(expr: TypedExpr) -> Self {
        Self::TypedExpr(Box::new(expr))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Cast {
    pub type_to: Box<Type>,
    pub type_orignal: Box<Type>,
    pub expr: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Comma {
    pub assigns: Vec<TypedExpr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum SemaExpr {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Char(char),
    String(Vec<char>),
    Symbol(Symbol),
    NumInt(usize),
    NumFloat(OrderedFloat<f64>),
    Subscript(Subscript),
    MemberAccess(MemberAccess),
    Ternary(Ternary),
    Unary(Unary),
    Sizeof(Sizeof),
    Cast(Cast),
    Comma(Comma),
}
impl SemaExpr {
    pub fn num_int(n: usize) -> Self {
        SemaExpr::NumInt(n)
    }

    pub fn num_float(n: OrderedFloat<f64>) -> Self {
        SemaExpr::NumFloat(n)
    }

    pub fn char_lit(c: char) -> Self {
        SemaExpr::Char(c)
    }

    pub fn string(string: Vec<char>) -> Self {
        SemaExpr::String(string)
    }

    pub fn ident(name: Symbol) -> Self {
        SemaExpr::Symbol(name)
    }

    pub fn unary(op: UnaryOp, expr: TypedExpr) -> Self {
        SemaExpr::Unary(Unary {
            op,
            expr: Box::new(expr),
        })
    }

    pub fn binary(op: BinaryOp, lhs: TypedExpr, rhs: TypedExpr) -> Self {
        SemaExpr::Binary(Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    pub fn ternary(cond: TypedExpr, then_branch: TypedExpr, else_branch: TypedExpr) -> Self {
        SemaExpr::Ternary(Ternary {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    pub fn assign(op: AssignOp, lhs: TypedExpr, rhs: TypedExpr) -> Self {
        SemaExpr::Assign(Assign {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    pub fn call(func: TypedExpr, args: Vec<TypedExpr>) -> Self {
        SemaExpr::Call(Call {
            func: Box::new(func),
            args: args.into_iter().collect(),
        })
    }

    pub fn subscript(subject: TypedExpr, index: TypedExpr) -> Self {
        SemaExpr::Subscript(Subscript {
            subject: Box::new(subject),
            index: Box::new(index),
        })
    }

    pub fn member_access(base: TypedExpr, member: Ident, kind: MemberAccessOp) -> Self {
        SemaExpr::MemberAccess(MemberAccess {
            base: Box::new(base),
            member,
            kind,
        })
    }

    pub fn sizeof(sizeof: Sizeof) -> Self {
        SemaExpr::Sizeof(sizeof)
    }

    pub fn cast(r#type_to: Type, r#type_orinal: Type, expr: TypedExpr) -> Self {
        SemaExpr::Cast(Cast {
            type_to: Box::new(r#type_to),
            type_orignal: Box::new(r#type_orinal),
            expr: Box::new(expr),
        })
    }

    pub fn comma(assigns: Vec<TypedExpr>) -> Self {
        SemaExpr::Comma(Comma { assigns })
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypedExpr {
    pub r#type: Type,
    pub r#expr: SemaExpr,
}

impl TypedExpr {
    pub fn new(r#type: Type, expr: SemaExpr) -> Self {
        Self {
            r#type,
            r#expr: expr,
        }
    }
}
