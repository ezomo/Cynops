use super::{
    Block, Expr, Ident,
    types::{FunctionSig, Type},
};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum DeclStmt {
    MemberDecl(MemberDecl),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef(Typedef),
}
impl DeclStmt {
    pub fn member_decl(m: MemberDecl) -> Self {
        DeclStmt::MemberDecl(m)
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
pub struct Typedef {
    pub ident: Ident,
    pub ty: Type,
}
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionDef {
    pub sig: FunctionSig,
    pub param_name: Vec<Ident>,
    pub body: Block,
}
