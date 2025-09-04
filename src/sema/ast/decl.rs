use crate::sema::ast::Symbol;

use super::TypedExpr;

use super::{
    Block, Ident, Typedef,
    types::{FunctionSig, Type},
};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum DeclStmt {
    InitVec(Vec<Init>),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef,
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
    pub ty: Type,
}

impl MemberDecl {
    pub fn new(ident: Symbol, ty: Type) -> Self {
        MemberDecl { sympl: ident, ty }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Struct {
    pub ident: Option<Ident>,
    pub member: Vec<MemberDecl>,
}

impl Struct {
    pub fn new(ident: Option<Ident>, member: Vec<MemberDecl>) -> Self {
        Struct { ident, member }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Union {
    pub ident: Option<Ident>,
    pub member: Vec<MemberDecl>,
}

impl Union {
    pub fn new(ident: Option<Ident>, member: Vec<MemberDecl>) -> Self {
        Union { ident, member }
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
    pub ident: Ident,
    pub value: Option<usize>, // 明示的な値がある場合
}

impl EnumMember {
    pub fn new(ident: Ident, value: Option<usize>) -> Self {
        EnumMember { ident, value }
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

impl InitData {
    pub fn as_compound(&self) -> Option<&Vec<InitData>> {
        if let Self::Compound(v) = self {
            Some(v)
        } else {
            None
        }
    }
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
