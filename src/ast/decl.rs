use super::{
    Block, Expr, Ident,
    types::{FunctionSig, Type},
};

#[derive(Debug, PartialEq, Clone)]
pub enum DeclStmt {
    Typed(Typed),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef(Typedef),
}

impl DeclStmt {
    pub fn typed(ty: Type, declarators: Vec<InitDeclarator>) -> Self {
        DeclStmt::Typed(Typed { ty, declarators })
    }

    pub fn struct_decl(s: Struct) -> Self {
        DeclStmt::Struct(s)
    }

    pub fn union_decl(u: Union) -> Self {
        DeclStmt::Union(u)
    }

    pub fn enum_decl(e: Enum) -> Self {
        DeclStmt::Enum(e)
    }

    pub fn typedef_decl(t: Typedef) -> Self {
        DeclStmt::Typedef(t)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Typed {
    pub ty: Type,
    pub declarators: Vec<InitDeclarator>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub init: Option<Initializer>,
}
impl InitDeclarator {
    pub fn new(declarator: Declarator, init: Option<Initializer>) -> Self {
        InitDeclarator { declarator, init }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pointer {
    pub level: usize,
    pub inner: Box<DirectDeclarator>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Declarator {
    Pointer(Pointer),
    Direct(DirectDeclarator),
}
impl Declarator {
    pub fn pointer(level: usize, inner: DirectDeclarator) -> Self {
        Declarator::Pointer(Pointer {
            level,
            inner: Box::new(inner),
        })
    }

    pub fn direct(direct: DirectDeclarator) -> Self {
        Declarator::Direct(direct)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DirectDeclarator {
    Ident(Ident),
    Paren(Box<Declarator>), // 例: (*f)
    Array {
        base: Box<DirectDeclarator>,
        size: Option<Expr>,
    },
    Func {
        base: Box<DirectDeclarator>,
        params: Option<ParamList>,
    },
}
impl DirectDeclarator {
    /// 識別子からDirectDeclaratorを作る
    pub fn ident(name: Ident) -> Self {
        DirectDeclarator::Ident(name)
    }

    /// 括弧つきDeclaratorからDirectDeclaratorを作る
    pub fn paren(decl: Declarator) -> Self {
        DirectDeclarator::Paren(Box::new(decl))
    }

    /// 配列型DirectDeclaratorを作る
    pub fn array(base: DirectDeclarator, size: Option<Expr>) -> Self {
        DirectDeclarator::Array {
            base: Box::new(base),
            size,
        }
    }

    /// 関数型DirectDeclaratorを作る
    pub fn func(base: DirectDeclarator, params: Option<ParamList>) -> Self {
        DirectDeclarator::Func {
            base: Box::new(base),
            params,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: Option<Declarator>,
}
impl Param {
    pub fn new(ty: Type, name: Option<Declarator>) -> Self {
        Self { ty, name: name }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParamList {
    Void,               // f(void)
    Params(Vec<Param>), // f(int x, char y)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Initializer {
    Expr(Box<Expr>),
    List(Vec<Initializer>), // 複合初期化子: {1, 2}
}
impl Initializer {
    /// 単一式による初期化子を作る
    pub fn expr(expr: Expr) -> Self {
        Initializer::Expr(Box::new(expr))
    }

    /// 複合リストによる初期化子を作る
    pub fn list(list: Vec<Initializer>) -> Self {
        Initializer::List(list)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDef {
    pub sig: FunctionSig,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub name: Option<Ident>,
    pub members: Vec<MemberDecl>,
}
impl Struct {
    pub fn new(name: Option<Ident>, members: Vec<MemberDecl>) -> Self {
        Self { name, members }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Union {
    pub name: Option<Ident>,
    pub members: Vec<MemberDecl>,
}
impl Union {
    pub fn new(name: Option<Ident>, members: Vec<MemberDecl>) -> Self {
        Self { name, members }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MemberDecl {
    pub ty: Type,
    pub declarators: Vec<Declarator>,
}
impl MemberDecl {
    pub fn new(ty: Type, declarators: Vec<Declarator>) -> Self {
        Self { ty, declarators }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: Option<Ident>,
    pub variants: Vec<EnumMember>,
}
impl Enum {
    pub fn new(name: Option<Ident>, variants: Vec<EnumMember>) -> Self {
        Self { name, variants }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: Ident,
    pub value: Option<usize>, // = 指定があるとき
}
impl EnumMember {
    pub fn new(name: Ident, value: Option<usize>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    pub ty: TypedefType,              // struct/union/enum/primitive など
    pub declarators: Vec<Declarator>, // Point, MyInt などの名前
}
impl Typedef {
    pub fn new(ty: TypedefType, declarators: Vec<Declarator>) -> Self {
        Self { ty, declarators }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedefType {
    Type(Type),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}
impl TypedefType {
    pub fn r#type(ty: Type) -> Self {
        TypedefType::Type(ty)
    }
    pub fn struct_decl(s: Struct) -> Self {
        TypedefType::Struct(s)
    }
    pub fn union_decl(u: Union) -> Self {
        TypedefType::Union(u)
    }
    pub fn enum_decl(e: Enum) -> Self {
        TypedefType::Enum(e)
    }
}
