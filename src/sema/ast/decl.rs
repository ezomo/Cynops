use super::TypedExpr;
use super::{Block, Ident, ScopePar, types::FunctionSig};
use crate::sema::ast::Symbol;
use std::hash::Hash;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum DeclStmt {
    InitVec(Vec<Init>),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef(Symbol),
}
impl DeclStmt {
    pub fn init_vec(vec: Vec<Init>) -> Self {
        DeclStmt::InitVec(vec)
    }

    pub fn r#struct(strct: Struct) -> Self {
        DeclStmt::Struct(strct)
    }

    pub fn union(union: Union) -> Self {
        DeclStmt::Union(union)
    }

    pub fn r#enum(enm: Enum) -> Self {
        DeclStmt::Enum(enm)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct MemberDecl {
    pub sympl: Symbol,
}

impl MemberDecl {
    pub fn new(symbol: Symbol) -> Self {
        MemberDecl { sympl: symbol }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Struct {
    pub ident: Option<Ident>,
    pub symbol: Symbol,
    pub member: Vec<MemberDecl>,
}

impl Struct {
    pub fn new(ident: Option<Ident>, symbol: Symbol, member: Vec<MemberDecl>) -> Self {
        Struct {
            ident,
            symbol,
            member,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Union {
    pub ident: Option<Ident>,
    pub symbol: Symbol,
    pub member: Vec<MemberDecl>,
}

impl Union {
    pub fn new(ident: Option<Ident>, symbol: Symbol, member: Vec<MemberDecl>) -> Self {
        Union {
            ident,
            symbol,
            member,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Enum {
    pub ident: Option<Ident>,
    pub variants: Vec<EnumMember>,
}

impl Enum {
    pub fn new(ident: Option<Ident>, variants: Vec<EnumMember>) -> Self {
        Enum { ident, variants }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct EnumMember {
    pub symbol: Symbol,
    pub value: Option<usize>, // 明示的な値がある場合
}

impl EnumMember {
    pub fn new(ident: Symbol, value: Option<usize>) -> Self {
        EnumMember {
            symbol: ident,
            value,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionDef {
    pub sig: FunctionSig,
    pub param_names: Vec<Symbol>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum InitData {
    Expr(TypedExpr),
    Compound(Vec<InitData>), // 構造体・配列初期化子 {1, 2}
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Init {
    pub r: MemberDecl,
    pub l: Option<InitData>,
}

impl Init {
    pub fn new(r: MemberDecl, l: Option<InitData>) -> Self {
        Init { r, l }
    }
}
