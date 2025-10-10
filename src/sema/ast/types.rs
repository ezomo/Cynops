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

#[derive(Debug, Clone, Eq)]
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

pub trait Size {
    fn size(&self) -> usize;
}

impl Size for Type {
    fn size(&self) -> usize {
        match self {
            Type::Void => 0,                // void型はサイズなし
            Type::Error => 0,               // エラー型はサイズなし
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
            Type::Typedef(this) => this.size(),
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
        self.member
            .iter()
            .map(|x| x.sympl.get_type().unwrap().size())
            .sum()
    }
}

impl Size for Union {
    fn size(&self) -> usize {
        self.member
            .iter()
            .map(|x| x.sympl.get_type().unwrap().size())
            .max()
            .unwrap_or(0)
    }
}

impl Size for Enum {
    fn size(&self) -> usize {
        Type::Int.size() // enumはintと同じサイズ
    }
}

impl Size for Symbol {
    fn size(&self) -> usize {
        self.get_type().unwrap().size()
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionSig {
    pub ty: Type,
    pub ident: Ident,
    pub scope_ptr: ScopePtr,
}
impl FunctionSig {
    pub fn new(ty: Type, ident: Ident, scope_ptr: ScopePtr) -> Self {
        FunctionSig {
            ty,
            ident,
            scope_ptr,
        }
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
        Self::new(format!("type..{}{}", self.to_string(), add.into()))
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
