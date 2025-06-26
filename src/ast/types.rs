use super::Declarator;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Void,
    Int,
    Char,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSig {
    pub ret_type: Type,
    pub declarator: Declarator,
}
impl FunctionSig {
    pub fn new(ret_type: Type, declarator: Declarator) -> Self {
        FunctionSig {
            ret_type,
            declarator,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ident {
    pub name: String,
}
impl Ident {
    /// 新しく Ident を作る（&str, String 両方対応）
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub name: Ident,
    pub members: Vec<StructMember>,
}
impl Struct {
    pub fn new(name: Ident, members: Vec<StructMember>) -> Self {
        Self { name, members }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructMember {
    pub ty: Type,
    pub declarators: Vec<Declarator>,
}

impl StructMember {
    pub fn new(ty: Type, declarators: Vec<Declarator>) -> Self {
        Self { ty, declarators }
    }
}
