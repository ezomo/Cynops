use super::*;
use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub array_of: Box<Type>,
    pub length: Option<Box<TypedExpr>>,
}

impl Array {
    pub fn new(array_of: Type, length: Option<TypedExpr>) -> Self {
        Self {
            array_of: Box::new(array_of),
            length: length.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct Func {
    pub return_type: Box<Type>,
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
    DotDotDot,
    Func(Func),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Pointer(Box<Type>),
    Array(Array),
    Typedef(Ident),
    Unresolved, //後でなくすかも
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
    /// Type を Rust 風の表記に変換する
    pub fn to_rust_format(&self) -> String {
        match self {
            Type::DotDotDot => "...".to_string(),
            Type::Void => "void".to_string(),
            Type::Int => "int".to_string(),
            Type::Double => "double".to_string(),
            Type::Char => "char".to_string(),
            Type::Unresolved => "unresolved".to_string(),
            Type::Func(func) => {
                let params = if func.params.is_empty() {
                    "void".to_string()
                } else {
                    func.params
                        .iter()
                        .map(|p| p.to_rust_format())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let return_type = func.return_type.to_rust_format();

                format!("fn({}) -> {}", params, return_type)
            }
            Type::Struct(s) => format!(
                "struct {}",
                s.ident
                    .as_ref()
                    .map_or("Anonymous".to_string(), |n| n.name.clone())
            ),
            Type::Union(u) => format!(
                "union {}",
                u.ident
                    .as_ref()
                    .map_or("Anonymous".to_string(), |n| n.name.clone())
            ),
            Type::Enum(e) => format!(
                "enum {}",
                e.ident
                    .as_ref()
                    .map_or("Anonymous".to_string(), |n| n.name.clone())
            ),
            Type::Pointer(inner) => {
                format!("*{}", inner.to_rust_format())
            }
            Type::Array(arr) => {
                format!(
                    "[{}; {}]",
                    arr.array_of.to_rust_format(),
                    arr.length
                        .as_ref()
                        .map_or("None".to_string(), |len| len.oneline())
                )
            }
            Type::Typedef(t) => t.to_string(),
        }
    }
}

impl Type {
    pub fn as_func(&self) -> Option<&Func> {
        if let Type::Func(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_struct(&self) -> Option<&Struct> {
        if let Type::Struct(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_union(&self) -> Option<&Union> {
        if let Type::Union(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_enum(&self) -> Option<&Enum> {
        if let Type::Enum(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_pointer(&self) -> Option<&Box<Type>> {
        if let Type::Pointer(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_array(&self) -> Option<&Array> {
        if let Type::Array(v) = self {
            Some(v)
        } else {
            None
        }
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

impl ast::Ident {
    /// ASTのIdentをcrate内共通型に変換
    pub fn as_same(&self) -> Ident {
        Ident {
            name: self.name.clone(),
        }
    }
}

impl Ident {
    /// 新しく Ident を作る（&str, String 両方対応）
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
