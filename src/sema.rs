use std::collections::HashMap;

// 元のAST定義からインポート（想定）
use crate::{
    ast::{
        Array as AstArray, DeclStmt, Declarator, DirectDeclarator, Enum, EnumMember,
        Func as AstFunc, FunctionSig, Ident, InitDeclarator, MemberDecl, Param, ParamList, Program,
        Stmt, Struct, TopLevel, Type as AstType, Typed, Typedef as AstTypedef, TypedefType, Union,
    },
    const_eval::eval_const_expr,
};

// 簡易型システムの定義
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub array_of: Box<Type>,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub return_type: Option<Box<Type>>,
    pub params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    pub type_name: Ident,
    pub actual_type: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Double,
    Char,
    Func(Func),
    Struct(Vec<Type>),
    Union(Vec<Type>),
    Typedef(Typedef),
    Enum(Vec<Ident>),
    Pointer(Box<Type>),
    Array(Array),
}

// 型抽出器
pub struct TypeExtractor {
    // typedef定義を保持
    typedef_map: HashMap<String, Type>,
    // struct/union/enum定義を保持
    struct_map: HashMap<String, Vec<Type>>,
    union_map: HashMap<String, Vec<Type>>,
    enum_map: HashMap<String, Vec<Ident>>,
}

impl TypeExtractor {
    pub fn new() -> Self {
        Self {
            typedef_map: HashMap::new(),
            struct_map: HashMap::new(),
            union_map: HashMap::new(),
            enum_map: HashMap::new(),
        }
    }

    // 宣言文から型を抽出
    pub fn extract_from_decl(&mut self, decl: &DeclStmt) -> Vec<Type> {
        match decl {
            DeclStmt::Typed(typed) => self.extract_from_typed(typed),
            DeclStmt::Struct(s) => {
                let struct_type = self.extract_from_struct(s);
                if let Some(name) = &s.name {
                    self.struct_map.insert(
                        name.name.clone(),
                        if let Type::Struct(members) = &struct_type {
                            members.clone()
                        } else {
                            vec![]
                        },
                    );
                }
                vec![struct_type]
            }
            DeclStmt::Union(u) => {
                let union_type = self.extract_from_union(u);
                if let Some(name) = &u.name {
                    self.union_map.insert(
                        name.name.clone(),
                        if let Type::Union(members) = &union_type {
                            members.clone()
                        } else {
                            vec![]
                        },
                    );
                }
                vec![union_type]
            }
            DeclStmt::Enum(e) => {
                let enum_type = self.extract_from_enum(e);
                if let Some(name) = &e.name {
                    self.enum_map.insert(
                        name.name.clone(),
                        if let Type::Enum(variants) = &enum_type {
                            variants.clone()
                        } else {
                            vec![]
                        },
                    );
                }
                vec![enum_type]
            }
            DeclStmt::Typedef(t) => self.extract_from_typedef(t),
        }
    }

    // 型付き宣言から型を抽出
    fn extract_from_typed(&mut self, typed: &Typed) -> Vec<Type> {
        let base_type = self.convert_ast_type(&typed.ty);
        typed
            .declarators
            .iter()
            .map(|init_decl| self.apply_declarator(&base_type, &init_decl.declarator))
            .collect()
    }

    // 宣言子を適用して完全な型を構築
    fn apply_declarator(&self, base_type: &Type, declarator: &Declarator) -> Type {
        match declarator {
            Declarator::Direct(direct) => self.apply_direct_declarator(base_type, direct),
            Declarator::Pointer(pointer) => {
                let mut result = base_type.clone();
                // 内側の宣言子を先に適用
                if let Some(inner) = pointer.inner.as_ref() {
                    result = self.apply_direct_declarator(&result, inner);
                }
                // ポインタレベル分だけPointerで包む
                for _ in 0..pointer.level {
                    result = Type::Pointer(Box::new(result));
                }
                result
            }
        }
    }

    // 直接宣言子を適用
    fn apply_direct_declarator(&self, base_type: &Type, direct: &DirectDeclarator) -> Type {
        match direct {
            DirectDeclarator::Ident(_) => base_type.clone(),
            DirectDeclarator::Paren(inner) => self.apply_declarator(base_type, inner),
            DirectDeclarator::Array(array) => {
                let element_type = self.apply_direct_declarator(base_type, &array.base);
                // 配列サイズが指定されていない場合は0とする
                let length = array
                    .size
                    .as_ref()
                    .and_then(|e| match eval_const_expr(e).unwrap() {
                        crate::const_eval::ConstValue::Int(i) => Some(i as usize),
                        _ => Some(0),
                    }) // 式の評価は複雑なので仮に0
                    .unwrap_or(0);
                Type::Array(Array {
                    array_of: Box::new(element_type),
                    length,
                })
            }
            DirectDeclarator::Func(func) => {
                let return_type = if matches!(base_type, Type::Void) {
                    None
                } else {
                    Some(Box::new(base_type.clone()))
                };

                let params = match &func.params {
                    Some(ParamList::Void) => vec![],
                    Some(ParamList::Params(params)) => params
                        .iter()
                        .map(|param| {
                            let param_type = self.convert_ast_type(&param.ty);
                            if let Some(decl) = &param.name {
                                self.apply_declarator(&param_type, decl)
                            } else {
                                param_type
                            }
                        })
                        .collect(),
                    None => vec![],
                };

                let base_type = self.apply_direct_declarator(&Type::Void, &func.base);
                Type::Func(Func {
                    return_type,
                    params,
                })
            }
        }
    }

    // AST型を簡易型に変換（修正版）
    fn convert_ast_type(&self, ast_type: &AstType) -> Type {
        match ast_type {
            AstType::Void => Type::Void,
            AstType::Int => Type::Int,
            AstType::Double => Type::Double,
            AstType::Char => Type::Char,
            AstType::Struct(ident) => {
                if let Some(members) = self.struct_map.get(&ident.name) {
                    Type::Struct(members.clone())
                } else {
                    // 前方宣言の場合は空のstruct
                    Type::Struct(vec![])
                }
            }
            AstType::Union(ident) => {
                if let Some(members) = self.union_map.get(&ident.name) {
                    Type::Union(members.clone())
                } else {
                    Type::Union(vec![])
                }
            }
            AstType::Enum(ident) => {
                if let Some(variants) = self.enum_map.get(&ident.name) {
                    Type::Enum(variants.clone())
                } else {
                    Type::Enum(vec![])
                }
            }
            AstType::Typedef(ident) => {
                // 修正: typedef名を解決して実際の型を返す
                if let Some(actual_type) = self.typedef_map.get(&ident.name) {
                    // 実際の型を返す（Typedef型でラップしない）
                    actual_type.clone()
                } else {
                    // 未定義のtypedefの場合はそのまま
                    Type::Typedef(Typedef {
                        type_name: ident.clone(),
                        actual_type: Box::new(Type::Void),
                    })
                }
            }
        }
    }

    // struct定義から型を抽出
    fn extract_from_struct(&mut self, s: &Struct) -> Type {
        let member_types = s
            .members
            .iter()
            .flat_map(|member| {
                let base_type = self.convert_ast_type(&member.ty);
                member
                    .declarators
                    .iter()
                    .map(|decl| self.apply_declarator(&base_type, decl))
                    .collect::<Vec<_>>()
            })
            .collect();
        Type::Struct(member_types)
    }

    // union定義から型を抽出
    fn extract_from_union(&mut self, u: &Union) -> Type {
        let member_types = u
            .members
            .iter()
            .flat_map(|member| {
                let base_type = self.convert_ast_type(&member.ty);
                member
                    .declarators
                    .iter()
                    .map(|decl| self.apply_declarator(&base_type, decl))
                    .collect::<Vec<_>>()
            })
            .collect();
        Type::Union(member_types)
    }

    // enum定義から型を抽出
    fn extract_from_enum(&self, e: &Enum) -> Type {
        let variants = e
            .variants
            .iter()
            .map(|variant| variant.name.clone())
            .collect();
        Type::Enum(variants)
    }

    // typedef定義から型を抽出（修正版）
    fn extract_from_typedef(&mut self, t: &AstTypedef) -> Vec<Type> {
        let base_type = match &t.ty {
            TypedefType::Type(ty) => self.convert_ast_type(ty),
            TypedefType::Struct(s) => self.extract_from_struct(s),
            TypedefType::Union(u) => self.extract_from_union(u),
            TypedefType::Enum(e) => self.extract_from_enum(e),
        };

        t.declarators
            .iter()
            .map(|decl| {
                let final_type = self.apply_declarator(&base_type, decl);
                // typedef名を抽出してマップに登録
                let name = self.extract_ident_from_declarator(decl.clone());

                // 修正: typedef_mapに実際の型を登録
                self.typedef_map
                    .insert(name.name.clone(), final_type.clone());

                // 戻り値としてはTypedef型を返す（デバッグ用）
                Type::Typedef(Typedef {
                    type_name: name.clone(),
                    actual_type: Box::new(final_type),
                })
            })
            .collect()
    }

    // 宣言子から識別子を抽出
    fn extract_ident_from_declarator(&self, declarator: Declarator) -> Ident {
        match declarator {
            Declarator::Pointer(p) => {
                if let Some(inner) = p.inner.as_ref() {
                    self.extract_ident_from_direct_declarator(inner.clone())
                } else {
                    // ポインタのみの場合は仮の識別子
                    Ident::new("unnamed")
                }
            }
            Declarator::Direct(di) => self.extract_ident_from_direct_declarator(di),
        }
    }

    fn extract_ident_from_direct_declarator(&self, directdeclarator: DirectDeclarator) -> Ident {
        match directdeclarator {
            DirectDeclarator::Array(a) => self.extract_ident_from_direct_declarator(*a.base),
            DirectDeclarator::Func(f) => self.extract_ident_from_direct_declarator(*f.base),
            DirectDeclarator::Paren(p) => self.extract_ident_from_declarator(*p),
            DirectDeclarator::Ident(i) => i,
        }
    }

    // 関数シグネチャから型を抽出
    pub fn extract_from_function_sig(&mut self, sig: &FunctionSig) -> Type {
        let return_type = self.convert_ast_type(&sig.ret_type);
        self.apply_declarator(&return_type, &sig.declarator)
    }
}

// 使用例
#[cfg(test)]
mod tests {
    use crate::{ast, ast_visualizer::visualize_program, lexer, parser, preprocessor};

    use super::*;

    #[test]
    fn test_basic_types() {
        let mut input = "typedef struct test { int x; int *y; } Test; Test g[10];".to_string();
        preprocessor::remove_comments(&mut input);
        let token = lexer::tokenize(&input);
        let p: ast::Program = parser::program(&mut parser::ParseSession::new(token));
        program(p.clone());
        visualize_program(&p);
    }
}

fn stmt(stmt: Stmt, extractor: &mut TypeExtractor) {
    match stmt {
        Stmt::DeclStmt(decl) => {
            let types = extractor.extract_from_decl(&decl);
            println!("{:#?}", types);
        }
        _ => todo!(),
    }
}

fn top_level(top_level: TopLevel, extractor: &mut TypeExtractor) {
    match top_level {
        TopLevel::FunctionDef(function_def) => todo!(),
        TopLevel::Stmt(stm) => stmt(stm, extractor),
        TopLevel::FunctionProto(_) => todo!(), // 関数プロトタイプは無視
    }
}

pub fn program(program: Program) {
    let mut extractor = TypeExtractor::new();

    for item in program.items {
        top_level(item, &mut extractor);
    }
}
