use crate::ast::Expr;

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
    Typedef(Typedef),
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

    pub fn typedef(typedef: Typedef) -> Self {
        DeclStmt::Typedef(typedef)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct MemberDecl {
    pub ident: Ident,
    pub ty: Type,
}

impl MemberDecl {
    pub fn new(ident: Ident, ty: Type) -> Self {
        MemberDecl { ident, ty }
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
    pub param_name: Vec<Ident>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum InitData {
    Expr(Expr),
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
