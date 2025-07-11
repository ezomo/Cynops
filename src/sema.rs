use std::collections::HashMap;

// 元のAST定義からインポート（想定）
use crate::{
    ast::{
        Array as AstArray, Block, Control, DeclStmt, Declarator, DirectDeclarator, Enum,
        EnumMember, Func as AstFunc, FunctionSig, Ident, InitDeclarator, MemberDecl, Param,
        ParamList, Program, Stmt, Struct, SwitchCase, TopLevel, Type as AstType, Typed,
        Typedef as AstTypedef, TypedefType, Union,
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

// 型抽出器（スコープ対応版）
pub struct TypeExtractor {
    // typedef定義をスコープごとに管理
    typedef_stack: Vec<HashMap<String, Type>>,
    // struct/union/enum定義をスコープごとに管理
    struct_stack: Vec<HashMap<String, Vec<Type>>>,
    union_stack: Vec<HashMap<String, Vec<Type>>>,
    enum_stack: Vec<HashMap<String, Vec<Ident>>>,
}

impl TypeExtractor {
    pub fn new() -> Self {
        Self {
            typedef_stack: vec![HashMap::new()], // グローバルスコープで初期化
            struct_stack: vec![HashMap::new()],
            union_stack: vec![HashMap::new()],
            enum_stack: vec![HashMap::new()],
        }
    }

    // 新しいスコープを開始
    pub fn push_scope(&mut self) {
        self.typedef_stack.push(HashMap::new());
        self.struct_stack.push(HashMap::new());
        self.union_stack.push(HashMap::new());
        self.enum_stack.push(HashMap::new());
    }

    // 現在のスコープを終了
    pub fn pop_scope(&mut self) {
        if self.typedef_stack.len() > 1 {
            self.typedef_stack.pop();
        }
        if self.struct_stack.len() > 1 {
            self.struct_stack.pop();
        }
        if self.union_stack.len() > 1 {
            self.union_stack.pop();
        }
        if self.enum_stack.len() > 1 {
            self.enum_stack.pop();
        }
    }

    // typedef名を現在のスコープに登録
    fn register_typedef(&mut self, name: String, ty: Type) {
        if let Some(current_scope) = self.typedef_stack.last_mut() {
            current_scope.insert(name, ty);
        }
    }

    // struct名を現在のスコープに登録
    fn register_struct(&mut self, name: String, members: Vec<Type>) {
        if let Some(current_scope) = self.struct_stack.last_mut() {
            current_scope.insert(name, members);
        }
    }

    // union名を現在のスコープに登録
    fn register_union(&mut self, name: String, members: Vec<Type>) {
        if let Some(current_scope) = self.union_stack.last_mut() {
            current_scope.insert(name, members);
        }
    }

    // enum名を現在のスコープに登録
    fn register_enum(&mut self, name: String, variants: Vec<Ident>) {
        if let Some(current_scope) = self.enum_stack.last_mut() {
            current_scope.insert(name, variants);
        }
    }

    // typedef名を解決（スコープを逆順に検索）
    fn resolve_typedef(&self, name: &str) -> Option<Type> {
        for scope in self.typedef_stack.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    // struct名を解決（スコープを逆順に検索）
    fn resolve_struct(&self, name: &str) -> Option<Vec<Type>> {
        for scope in self.struct_stack.iter().rev() {
            if let Some(members) = scope.get(name) {
                return Some(members.clone());
            }
        }
        None
    }

    // union名を解決（スコープを逆順に検索）
    fn resolve_union(&self, name: &str) -> Option<Vec<Type>> {
        for scope in self.union_stack.iter().rev() {
            if let Some(members) = scope.get(name) {
                return Some(members.clone());
            }
        }
        None
    }

    // enum名を解決（スコープを逆順に検索）
    fn resolve_enum(&self, name: &str) -> Option<Vec<Ident>> {
        for scope in self.enum_stack.iter().rev() {
            if let Some(variants) = scope.get(name) {
                return Some(variants.clone());
            }
        }
        None
    }

    // 宣言文から型を抽出
    pub fn extract_from_decl(&mut self, decl: &DeclStmt) -> Vec<Type> {
        match decl {
            DeclStmt::Typed(typed) => self.extract_from_typed(typed),
            DeclStmt::Struct(s) => {
                let struct_type = self.extract_from_struct(s);
                if let Some(name) = &s.name {
                    if let Type::Struct(members) = &struct_type {
                        self.register_struct(name.name.clone(), members.clone());
                    }
                }
                vec![struct_type]
            }
            DeclStmt::Union(u) => {
                let union_type = self.extract_from_union(u);
                if let Some(name) = &u.name {
                    if let Type::Union(members) = &union_type {
                        self.register_union(name.name.clone(), members.clone());
                    }
                }
                vec![union_type]
            }
            DeclStmt::Enum(e) => {
                let enum_type = self.extract_from_enum(e);
                if let Some(name) = &e.name {
                    if let Type::Enum(variants) = &enum_type {
                        self.register_enum(name.name.clone(), variants.clone());
                    }
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
                    })
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

    // AST型を簡易型に変換（スコープ対応版）
    fn convert_ast_type(&self, ast_type: &AstType) -> Type {
        match ast_type {
            AstType::Void => Type::Void,
            AstType::Int => Type::Int,
            AstType::Double => Type::Double,
            AstType::Char => Type::Char,
            AstType::Struct(ident) => {
                if let Some(members) = self.resolve_struct(&ident.name) {
                    Type::Struct(members)
                } else {
                    // 前方宣言の場合は空のstruct
                    Type::Struct(vec![])
                }
            }
            AstType::Union(ident) => {
                if let Some(members) = self.resolve_union(&ident.name) {
                    Type::Union(members)
                } else {
                    Type::Union(vec![])
                }
            }
            AstType::Enum(ident) => {
                if let Some(variants) = self.resolve_enum(&ident.name) {
                    Type::Enum(variants)
                } else {
                    Type::Enum(vec![])
                }
            }
            AstType::Typedef(ident) => {
                // スコープを考慮してtypedef名を解決
                if let Some(actual_type) = self.resolve_typedef(&ident.name) {
                    actual_type
                } else {
                    // 未定義のtypedefの場合
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

    // typedef定義から型を抽出（スコープ対応版）
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
                // typedef名を抽出してスコープに登録
                let name = self.extract_ident_from_declarator(decl.clone());

                // 現在のスコープにtypedef名を登録
                self.register_typedef(name.name.clone(), final_type.clone());

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

    // ブロック文を処理（スコープ管理）
    pub fn process_block(&mut self, block: &Block) {
        self.push_scope();

        for stmt in &block.statements {
            self.process_stmt(stmt);
        }

        self.pop_scope();
    }

    // 文を処理（スコープを考慮）
    pub fn process_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::DeclStmt(decl) => {
                let types = self.extract_from_decl(decl);
                println!("Extracted types: {:#?}", types);
            }
            Stmt::Block(block) => {
                self.process_block(block);
            }
            Stmt::Control(control) => {
                // 制御文のネストしたブロックも処理
                match control {
                    Control::If(if_stmt) => {
                        self.process_stmt(&if_stmt.then_branch);
                        if let Some(else_branch) = &if_stmt.else_branch {
                            self.process_stmt(else_branch);
                        }
                    }
                    Control::While(while_stmt) => {
                        self.process_stmt(&while_stmt.body);
                    }
                    Control::DoWhile(do_while_stmt) => {
                        self.process_stmt(&do_while_stmt.body);
                    }
                    Control::For(for_stmt) => {
                        self.push_scope(); // forループは独自のスコープを持つ
                        self.process_stmt(&for_stmt.body);
                        self.pop_scope();
                    }
                    Control::Switch(switch_stmt) => {
                        for case in &switch_stmt.cases {
                            match case {
                                SwitchCase::Case(case) => {
                                    for stmt in &case.stmts {
                                        self.process_stmt(stmt);
                                    }
                                }
                                SwitchCase::Default(default) => {
                                    for stmt in &default.stmts {
                                        self.process_stmt(stmt);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Label(label) => {
                self.process_stmt(&label.stmt);
            }
            _ => {
                // その他の文は特別な処理不要
            }
        }
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

    #[test]
    fn test_scope_handling() {
        let mut input = r#"
        typedef int MyInt;
        {
            typedef double MyInt;  // 内側のスコープで再定義
            MyInt x;  // これはdouble型
        }
        MyInt y;  // これはint型
        "#
        .to_string();

        preprocessor::remove_comments(&mut input);
        let token = lexer::tokenize(&input);
        let p: ast::Program = parser::program(&mut parser::ParseSession::new(token));
        program(p.clone());
    }
}

fn stmt(stmt: Stmt, extractor: &mut TypeExtractor) {
    extractor.process_stmt(&stmt);
}

fn top_level(top_level: TopLevel, extractor: &mut TypeExtractor) {
    match top_level {
        TopLevel::FunctionDef(function_def) => {
            // 関数定義は新しいスコープを作成
            extractor.push_scope();

            // 関数のパラメータを現在のスコープに登録
            // （実装は関数の詳細に依存）

            // 関数本体を処理（BlockなのでProcess_blockを使用）
            extractor.process_block(&function_def.body);

            extractor.pop_scope();
        }
        TopLevel::Stmt(stm) => stmt(stm, extractor),
        TopLevel::FunctionProto(_) => {
            // 関数プロトタイプは型情報のみ処理
        }
    }
}

pub fn program(program: Program) {
    let mut extractor = TypeExtractor::new();

    for item in program.items {
        top_level(item, &mut extractor);
    }
}
