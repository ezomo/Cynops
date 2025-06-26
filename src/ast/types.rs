use super::Declarator;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Void,
    Int,
    Char,
    Struct(Ident), // struct Foo
    Union(Ident),  // union Bar
    Enum(Ident),   // enum Baz
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
