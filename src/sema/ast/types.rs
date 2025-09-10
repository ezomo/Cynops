use super::*;
use crate::ast;
use crate::visualize::*;

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
    pub fn pointer(p: Type) -> Self {
        Type::Pointer(Box::new(p))
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
            Type::Struct(s) => format!("struct {}", s.type_ident.to_string()),
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

    pub fn as_array(&self) -> Option<&Array> {
        if let Type::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

pub trait Size {
    fn size(&self) -> usize;
}

impl Size for Type {
    fn size(&self) -> usize {
        match self {
            Type::Void => 0,                // void型はサイズなし
            Type::Char => 1,                // char は 1 バイト
            Type::Int => 4,                 // int は 4 バイト (32bit想定)
            Type::Double => 8,              // double は 8 バイト
            Type::DotDotDot => 0,           // 可変長引数はサイズなし
            Type::Unresolved => 0,          // 未解決型はサイズ不明
            Type::Pointer(_) => 8,          // ポインタは 8 バイト (64bit想定)
            Type::Func(f) => f.size(),      // 関数型のサイズ取得
            Type::Array(arr) => arr.size(), // 配列のサイズ取得
            Type::Struct(s) => s.size(),    // 構造体のサイズ取得
            Type::Union(u) => u.size(),     // 共用体のサイズ取得
            Type::Enum(e) => e.size(),      // enumのサイズ取得
        }
    }
}

impl Size for Func {
    fn size(&self) -> usize {
        Type::Void.size() // 関数型は void と仮定
    }
}

impl Size for Array {
    fn size(&self) -> usize {
        let element_size = self.array_of.size();
        match &self.length {
            Some(len_expr) => element_size * len_expr.consume_const() as usize,
            None => panic!("Incomplete array type"),
        }
    }
}

impl Size for Struct {
    fn size(&self) -> usize {
        self.member.iter().map(|x| x.ty.size()).sum()
    }
}

impl Size for Union {
    fn size(&self) -> usize {
        self.member.iter().map(|x| x.ty.size()).max().unwrap_or(0)
    }
}

impl Size for Enum {
    fn size(&self) -> usize {
        Type::Int.size() // enumはintと同じサイズ
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

    pub fn with_suffix(&self, add: impl Into<String>) -> Self {
        Self::new(format!("d{}{}", self.to_string(), add.into()))
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
