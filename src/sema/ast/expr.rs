use super::{Ident, ScopeNode, Type};
use crate::op::*;
use ordered_float::OrderedFloat;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Weak;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct PostfixChain {
    pub base: TypedExpr,              // primary に相当する基の式
    pub suffixes: Vec<PostfixSuffix>, // 後置操作の連続
}

impl PostfixChain {
    pub fn new(base: TypedExpr, suffixes: Vec<PostfixSuffix>) -> Self {
        PostfixChain { base, suffixes }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum PostfixSuffix {
    ArrayAcsess(TypedExpr),              // [ expr ]
    ArgList(Vec<Box<TypedExpr>>),        // ( arg_list )
    MemberAccess(MemberAccessOp, Ident), // . ident または -> ident
}

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
    pub args: Vec<Box<TypedExpr>>,
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
    pub r#type: Box<Type>,
    pub expr: Box<TypedExpr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Comma {
    pub assigns: Vec<TypedExpr>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Symbol {
    pub ident: Ident,
    pub scope: ScopePar, // どこのスコープで定義されたか
}

impl Symbol {
    pub fn new(name: Ident, scope: Weak<RefCell<ScopeNode>>) -> Self {
        Symbol {
            ident: name,
            scope: ScopePar::new(scope),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScopePar {
    pub ptr: Weak<RefCell<ScopeNode>>,
}
impl ScopePar {
    pub fn new(ptr: Weak<RefCell<ScopeNode>>) -> Self {
        ScopePar { ptr }
    }
}

impl Hash for ScopePar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state);
    }
}

impl PartialEq for ScopePar {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl Eq for ScopePar {}

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

    pub fn unary(op: UnaryOp, expr: Box<TypedExpr>) -> Box<Self> {
        Box::new(SemaExpr::Unary(Unary { op, expr }))
    }

    pub fn binary(op: BinaryOp, lhs: Box<TypedExpr>, rhs: Box<TypedExpr>) -> Box<Self> {
        Box::new(SemaExpr::Binary(Binary { op, lhs, rhs }))
    }

    pub fn ternary(
        cond: Box<TypedExpr>,
        then_branch: TypedExpr,
        else_branch: TypedExpr,
    ) -> Box<Self> {
        Box::new(SemaExpr::Ternary(Ternary {
            cond,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }))
    }

    pub fn assign(op: AssignOp, lhs: Box<TypedExpr>, rhs: Box<TypedExpr>) -> Box<Self> {
        Box::new(SemaExpr::Assign(Assign { op, lhs, rhs }))
    }

    pub fn call(func: TypedExpr, args: Vec<Box<TypedExpr>>) -> Self {
        SemaExpr::Call(Call {
            func: Box::new(func),
            args,
        })
    }
    pub fn subscript(name: TypedExpr, index: TypedExpr) -> Self {
        SemaExpr::Subscript(Subscript {
            subject: Box::new(name),
            index: Box::new(index),
        })
    }

    pub fn member_access(base: TypedExpr, member: Ident, kind: MemberAccessOp) -> Self {
        SemaExpr::MemberAccess(MemberAccess {
            base: Box::new(base),
            member: member,
            kind,
        })
    }

    pub fn sizeof(sizeof: Sizeof) -> Box<Self> {
        Box::new(SemaExpr::Sizeof(sizeof))
    }

    pub fn cast(r#type: Type, expr: TypedExpr) -> Box<Self> {
        Box::new(SemaExpr::Cast(Cast {
            r#type: Box::new(r#type),
            expr: Box::new(expr),
        }))
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
