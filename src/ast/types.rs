use crate::ast::{Enum, Union};

use super::Struct;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub array_of: Box<Type>,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct Func {
    pub return_type: Option<Box<Type>>,
    pub params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct Typedef {
    pub type_name: Ident,
    pub actual_type: Box<Type>,
}
impl Typedef {
    pub fn new(type_name: Ident, actual_type: Type) -> Self {
        Typedef {
            type_name,
            actual_type: Box::new(actual_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Int,
    Double,
    Char,
    Func(Func),
    Struct(Struct),
    Union(Union),
    Typedef(Typedef),
    Enum(Enum),
    Pointer(Box<Type>),
    Array(Array),
}

impl Type {
    pub fn r#struct(s: Struct) -> Self {
        Type::Struct(s)
    }
    pub fn r#union(u: Union) -> Self {
        Type::Union(u)
    }
    pub fn r#enum(e: Enum) -> Self {
        Type::Enum(e)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionSig {
    pub ty: Type,
    pub ident: Ident,
}
impl FunctionSig {
    pub fn new(ty: Type, ident: Ident) -> Self {
        FunctionSig { ty, ident }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]

pub struct Ident {
    pub name: String,
}
impl Ident {
    /// 新しく Ident を作る（&str, String 両方対応）
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
