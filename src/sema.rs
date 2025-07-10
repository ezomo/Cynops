use crate::ast::Ident;

struct Array {
    array_of: Type,
    lenght: usize,
}

struct Func {
    return_type: Option<Box<Type>>,
    param: Vec<Type>,
}

struct Typedef {
    type_name: Ident,
    actual_type: Box<Type>,
}

enum Type {
    Int,
    Double,
    Char,
    Func(Func),
    Struct(Vec<Type>),
    Union(Vec<Type>),
    Typedef(Typedef),
    Enum(Vec<Ident>),
    Pointer(Box<Type>),
}
impl Type {
    fn pointer(ty: Self) -> Self {
        Self::Pointer(Box::new(ty))
    }
}
