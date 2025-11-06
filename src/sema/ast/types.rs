use super::*;
use crate::ast;
use crate::visualize::*;
use std::hash::Hash;

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

#[derive(Clone, Eq)]
pub enum Type {
    Void,
    Error, //エラー時に使う
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
    Typedef(Symbol),
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.flat().to_rust_format())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // 両方がTypedefの場合、実際の型を取得して比較
            (Type::Typedef(sym1), Type::Typedef(sym2)) => {
                match (sym1.get_type(), sym2.get_type()) {
                    (Some(ty1), Some(ty2)) => ty1.eq(&ty2),
                    _ => false,
                }
            }
            // 片方がTypedefの場合、Typedefの実際の型と比較
            (Type::Typedef(sym), other_type) => match sym.get_type() {
                Some(ty) => ty.eq(other_type),
                None => false,
            },
            (other_type, Type::Typedef(sym)) => match sym.get_type() {
                Some(ty) => other_type.eq(&ty),
                None => false,
            },
            // その他の場合は標準的な比較
            (Type::Void, Type::Void) => true,
            (Type::Int, Type::Int) => true,
            (Type::Double, Type::Double) => true,
            (Type::Char, Type::Char) => true,
            (Type::DotDotDot, Type::DotDotDot) => true,
            (Type::Unresolved, Type::Unresolved) => true,
            (Type::Func(f1), Type::Func(f2)) => f1.eq(f2),
            (Type::Struct(s1), Type::Struct(s2)) => s1.eq(s2),
            (Type::Union(u1), Type::Union(u2)) => u1.eq(u2),
            (Type::Enum(e1), Type::Enum(e2)) => e1.eq(e2),
            (Type::Pointer(p1), Type::Pointer(p2)) => p1.eq(p2),
            (Type::Array(a1), Type::Array(a2)) => a1.eq(a2),
            _ => false,
        }
    }
}

// Hashも再実装が必要（PartialEqを変更したため）
impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::Typedef(sym) => {
                // Typedefの場合は実際の型をハッシュ化
                if let Some(actual_type) = sym.get_type() {
                    actual_type.hash(state);
                } else {
                    // 型が解決できない場合は元の実装に従う
                    std::mem::discriminant(self).hash(state);
                    sym.hash(state);
                }
            }
            Type::Void => std::mem::discriminant(self).hash(state),
            Type::Error => std::mem::discriminant(self).hash(state),
            Type::Int => std::mem::discriminant(self).hash(state),
            Type::Double => std::mem::discriminant(self).hash(state),
            Type::Char => std::mem::discriminant(self).hash(state),
            Type::DotDotDot => std::mem::discriminant(self).hash(state),
            Type::Unresolved => std::mem::discriminant(self).hash(state),
            Type::Func(f) => {
                std::mem::discriminant(self).hash(state);
                f.hash(state);
            }
            Type::Struct(s) => {
                std::mem::discriminant(self).hash(state);
                s.hash(state);
            }
            Type::Union(u) => {
                std::mem::discriminant(self).hash(state);
                u.hash(state);
            }
            Type::Enum(e) => {
                std::mem::discriminant(self).hash(state);
                e.hash(state);
            }
            Type::Pointer(p) => {
                std::mem::discriminant(self).hash(state);
                p.hash(state);
            }
            Type::Array(a) => {
                std::mem::discriminant(self).hash(state);
                a.hash(state);
            }
        }
    }
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
            Type::Error => "error".to_string(),
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
            Type::Struct(s) => format!("{}", s.symbol.ident.to_string()),
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
            Type::Typedef(this) => this.get_type().unwrap().to_rust_format(),
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
    pub fn as_par(&self) -> Option<&Type> {
        if let Type::Pointer(ptr) = self {
            Some(ptr)
        } else {
            None
        }
    }
}

impl Type {
    pub fn flat(&self) -> Type {
        match self {
            Type::Typedef(sym) => {
                match sym.get_type() {
                    Some(actual_type) => actual_type.flat(), // 再帰的にflat化
                    None => Type::Unresolved,                // 型が解決できない場合
                }
            }
            // その他の型はそのまま返す（ただし内部のTypedefも再帰的にflat化）
            Type::Pointer(inner) => Type::Pointer(Box::new(inner.flat())),
            Type::Array(arr) => Type::Array(Array {
                array_of: Box::new(arr.array_of.flat()),
                length: arr.length.clone(),
            }),
            Type::Func(func) => Type::Func(Func {
                return_type: Box::new(func.return_type.flat()),
                params: func.params.iter().map(|p| p.flat()).collect(),
            }),
            // Struct, Union, Enumの内部メンバーのTypedefも解決する場合は以下のようにする
            // Type::Struct(s) => {
            //     // Structのメンバーも再帰的にflat化する場合
            //     Type::Struct(s.clone()) // 今回は簡単のためclone
            // } //循環参照して死ぬ？　TODO
            // その他の基本型はそのまま返す
            _ => self.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionSig {
    pub symbol: Symbol,
}
impl FunctionSig {
    pub fn new(symbol: Symbol) -> Self {
        FunctionSig { symbol }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]

pub struct Ident {
    pub name: String,
}

impl<T: Into<String>> From<T> for Ident {
    fn from(value: T) -> Self {
        Ident { name: value.into() }
    }
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

    pub fn with_suffix(&self, add: impl Into<String>) -> Self {
        format!("type..{}{}", self.to_string(), add.into()).into()
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
