use super::ast::*;

pub fn program(program: &Program, session: &mut Session) -> Program {
    let mut resolved_items = Vec::new();

    for item in &program.items {
        let resolved_item = resolve_toplevel(item, session);
        resolved_items.push(resolved_item);
    }

    Program {
        items: resolved_items,
    }
}

fn resolve_toplevel(toplevel: &TopLevel, session: &mut Session) -> TopLevel {
    match toplevel {
        TopLevel::FunctionDef(func_def) => {
            TopLevel::FunctionDef(resolve_function_def(func_def, session))
        }
        TopLevel::FunctionProto(func_proto) => {
            TopLevel::FunctionProto(func_proto.clone()) // プロトタイプは既に解決済み
        }
        TopLevel::Stmt(stmt) => TopLevel::Stmt(resolve_stmt(stmt, session)),
    }
}

fn resolve_function_def(func_def: &FunctionDef, session: &mut Session) -> FunctionDef {
    // 関数のスコープに入る
    session.push_scope();

    // パラメータを登録
    if let Some(func_type) = func_def.sig.ty.as_func() {
        for (param_name, param_type) in func_def.param_names.iter().zip(&func_type.params) {
            session.register_symbols(param_name.clone(), param_type.clone());
        }
    }

    let resolved_body = resolve_block(&func_def.body, session);

    session.pop_scope();

    FunctionDef {
        sig: func_def.sig.clone(),
        param_names: func_def.param_names.clone(),
        body: resolved_body,
    }
}

fn resolve_block(block: &Block, session: &mut Session) -> Block {
    session.push_scope();

    let mut resolved_statements = Vec::new();
    for stmt in &block.statements {
        let resolved_stmt = resolve_stmt(stmt, session);
        resolved_statements.push(Box::new(resolved_stmt));
    }

    session.pop_scope();

    Block {
        statements: resolved_statements,
    }
}

fn resolve_stmt(stmt: &Stmt, session: &mut Session) -> Stmt {
    match stmt {
        Stmt::ExprStmt(expr) => Stmt::ExprStmt(resolve_typed_expr(expr, session)),
        Stmt::DeclStmt(decl) => Stmt::DeclStmt(resolve_decl_stmt(decl, session)),
        Stmt::Control(control) => Stmt::Control(resolve_control(control, session)),
        Stmt::Return(ret) => Stmt::Return(Return {
            value: ret
                .value
                .as_ref()
                .map(|v| Box::new(resolve_typed_expr(v, session))),
        }),
        Stmt::Block(block) => Stmt::Block(resolve_block(block, session)),
        Stmt::Label(label) => Stmt::Label(Label {
            name: label.name.clone(),
            stmt: Box::new(resolve_stmt(&label.stmt, session)),
        }),
        _ => stmt.clone(), // Goto, Break, Continue はそのまま
    }
}

fn resolve_control(control: &Control, session: &mut Session) -> Control {
    match control {
        Control::If(if_stmt) => Control::If(If {
            cond: Box::new(resolve_typed_expr(&if_stmt.cond, session)),
            then_branch: Box::new(resolve_stmt(&if_stmt.then_branch, session)),
            else_branch: if_stmt
                .else_branch
                .as_ref()
                .map(|e| Box::new(resolve_stmt(e, session))),
        }),
        Control::While(while_stmt) => Control::While(While {
            cond: Box::new(resolve_typed_expr(&while_stmt.cond, session)),
            body: Box::new(resolve_stmt(&while_stmt.body, session)),
        }),
        Control::DoWhile(do_while) => Control::DoWhile(DoWhile {
            body: Box::new(resolve_stmt(&do_while.body, session)),
            cond: Box::new(resolve_typed_expr(&do_while.cond, session)),
        }),
        Control::For(for_stmt) => Control::For(For {
            init: for_stmt
                .init
                .as_ref()
                .map(|i| Box::new(resolve_typed_expr(i, session))),
            cond: for_stmt
                .cond
                .as_ref()
                .map(|c| Box::new(resolve_typed_expr(c, session))),
            step: for_stmt
                .step
                .as_ref()
                .map(|s| Box::new(resolve_typed_expr(s, session))),
            body: Box::new(resolve_stmt(&for_stmt.body, session)),
        }),
        Control::Switch(switch) => Control::Switch(Switch {
            cond: Box::new(resolve_typed_expr(&switch.cond, session)),
            cases: switch
                .cases
                .iter()
                .map(|c| resolve_switch_case(c, session))
                .collect(),
        }),
    }
}

fn resolve_switch_case(case: &SwitchCase, session: &mut Session) -> SwitchCase {
    match case {
        SwitchCase::Case(c) => SwitchCase::Case(Case {
            const_expr: resolve_typed_expr(&c.const_expr, session),
            stmts: c
                .stmts
                .iter()
                .map(|s| Box::new(resolve_stmt(s, session)))
                .collect(),
        }),
        SwitchCase::Default(d) => SwitchCase::Default(DefaultCase {
            stmts: d
                .stmts
                .iter()
                .map(|s| Box::new(resolve_stmt(s, session)))
                .collect(),
        }),
    }
}

fn resolve_decl_stmt(decl: &DeclStmt, session: &mut Session) -> DeclStmt {
    match decl {
        DeclStmt::InitVec(inits) => {
            let mut resolved_inits = Vec::new();
            for init in inits {
                let resolved_init = resolve_init(init, session);
                resolved_inits.push(resolved_init);
            }
            DeclStmt::InitVec(resolved_inits)
        }
        _ => decl.clone(), // Struct, Union, Enum, Typedef はconvert時点で解決済み
    }
}

fn resolve_init(init: &Init, session: &mut Session) -> Init {
    let mut resolved_type = init.r.ty.clone();

    // 配列の長さ推論
    if let Some(init_data) = &init.l {
        infer_array_length(&mut resolved_type, init_data, session);
    }

    let resolved_member_decl = MemberDecl {
        ident: init.r.ident.clone(),
        ty: resolved_type.clone(),
    };

    // 変数をセッションに登録
    session.register_symbols(
        resolved_member_decl.ident.clone(),
        resolved_member_decl.ty.clone(),
    );

    Init {
        r: resolved_member_decl,
        l: init.l.as_ref().map(|data| resolve_init_data(data, session)),
    }
}

fn infer_array_length(array_type: &mut Type, init_data: &InitData, session: &mut Session) {
    if let Type::Array(array) = array_type {
        // 長さが未定義の場合
        if array.length.is_none() {
            if let InitData::Compound(compounds) = init_data {
                // 初期化子の長さを TypedExpr に変換
                let len_expr = TypedExpr::new(
                    Type::Int,                         // 配列長は整数型
                    SemaExpr::NumInt(compounds.len()), // 要素数
                );
                array.length = Some(Box::new(len_expr));
            }
        } else {
            let len_expr = TypedExpr::new(
                Type::Int, // 配列長は整数型
                SemaExpr::NumInt(
                    resolve_typed_expr(array.length.clone().unwrap().as_ref(), session)
                        .eval_const()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                ), // 要素数
            );
            array.length = Some(Box::new(len_expr));
        }

        // 配列の長さが式で指定されている場合の定数評価もここで可能
        // 例: const folding など
    }
}
fn resolve_init_data(data: &InitData, session: &mut Session) -> InitData {
    match data {
        InitData::Expr(expr) => InitData::Expr(resolve_typed_expr(expr, session)),
        InitData::Compound(compounds) => InitData::Compound(
            compounds
                .iter()
                .map(|c| resolve_init_data(c, session))
                .collect(),
        ),
    }
}

fn resolve_typed_expr(expr: &TypedExpr, session: &mut Session) -> TypedExpr {
    let resolved_sema_expr = resolve_sema_expr(&expr.r#expr, session);
    let inferred_type = infer_type(&resolved_sema_expr, session);

    TypedExpr {
        r#type: inferred_type,
        r#expr: resolved_sema_expr,
    }
}

fn resolve_sema_expr(expr: &SemaExpr, session: &mut Session) -> SemaExpr {
    match expr {
        SemaExpr::Assign(assign) => SemaExpr::Assign(Assign {
            op: assign.op,
            lhs: Box::new(resolve_typed_expr(&assign.lhs, session)),
            rhs: Box::new(resolve_typed_expr(&assign.rhs, session)),
        }),
        SemaExpr::Binary(binary) => SemaExpr::Binary(Binary {
            op: binary.op,
            lhs: Box::new(resolve_typed_expr(&binary.lhs, session)),
            rhs: Box::new(resolve_typed_expr(&binary.rhs, session)),
        }),
        SemaExpr::Unary(unary) => SemaExpr::Unary(Unary {
            op: unary.op,
            expr: Box::new(resolve_typed_expr(&unary.expr, session)),
        }),
        SemaExpr::Ternary(ternary) => SemaExpr::Ternary(Ternary {
            cond: Box::new(resolve_typed_expr(&ternary.cond, session)),
            then_branch: Box::new(resolve_typed_expr(&ternary.then_branch, session)),
            else_branch: Box::new(resolve_typed_expr(&ternary.else_branch, session)),
        }),
        SemaExpr::Call(call) => SemaExpr::Call(Call {
            func: Box::new(resolve_typed_expr(&call.func, session)),
            args: call
                .args
                .iter()
                .map(|a| Box::new(resolve_typed_expr(a, session)))
                .collect(),
        }),
        SemaExpr::Subscript(subscript) => SemaExpr::Subscript(Subscript {
            subject: Box::new(resolve_typed_expr(&subscript.subject, session)),
            index: Box::new(resolve_typed_expr(&subscript.index, session)),
        }),
        SemaExpr::MemberAccess(member) => SemaExpr::MemberAccess(MemberAccess {
            base: Box::new(resolve_typed_expr(&member.base, session)),
            member: member.member.clone(),
            kind: member.kind.clone(),
        }),
        SemaExpr::Cast(cast) => SemaExpr::Cast(Cast {
            r#type: cast.r#type.clone(),
            expr: Box::new(resolve_typed_expr(&cast.expr, session)),
        }),
        SemaExpr::Comma(comma) => SemaExpr::Comma(Comma {
            assigns: comma
                .assigns
                .iter()
                .map(|a| resolve_typed_expr(a, session))
                .collect(),
        }),
        SemaExpr::Sizeof(sizeof) => {
            let resolved_sizeof = match sizeof {
                Sizeof::Type(ty) => Sizeof::Type(ty.clone()),
                Sizeof::TypedExpr(expr) => {
                    Sizeof::TypedExpr(Box::new(resolve_typed_expr(expr, session)))
                }
            };
            SemaExpr::Sizeof(resolved_sizeof)
        }
        _ => expr.clone(), // リテラル類やIdentはそのまま
    }
}

fn infer_type(expr: &SemaExpr, session: &mut Session) -> Type {
    match expr {
        SemaExpr::NumInt(_) => Type::Int,
        SemaExpr::NumFloat(_) => Type::Double,
        SemaExpr::Char(_) => Type::Char,
        SemaExpr::String(_) => {
            // 文字列リテラルは char の配列
            Type::Array(Array {
                array_of: Box::new(Type::Char),
                length: None, // 文字列長は実行時に決定
            })
        }
        SemaExpr::Ident(symbol) => {
            // シンボルテーブルから型を検索
            session
                .get_variable(&symbol.name)
                .unwrap_or_else(|| panic!("未定義の変数: {}", symbol.name.name))
        }
        SemaExpr::Binary(binary) => infer_binary_type(binary, session),
        SemaExpr::Unary(unary) => infer_unary_type(unary, session),
        SemaExpr::Assign(assign) => {
            // 代入式の型は左辺の型
            infer_type(&assign.lhs.r#expr, session)
        }
        SemaExpr::Call(call) => {
            // 関数呼び出しの戻り値型を推論
            let func_type = infer_type(&call.func.r#expr, session);
            if let Type::Func(func) = func_type {
                *func.return_type
            } else {
                panic!("関数でないものを呼び出そうとしています")
            }
        }
        SemaExpr::Subscript(subscript) => {
            // 配列添字の型は配列要素の型
            let subject_type = infer_type(&subscript.subject.r#expr, session);
            match subject_type {
                Type::Array(array) => *array.array_of,
                Type::Pointer(inner) => *inner,
                _ => panic!("配列またはポインタでないものに添字アクセスしています"),
            }
        }
        SemaExpr::MemberAccess(member) => {
            // 構造体メンバアクセスの型推論
            let base_type = infer_type(&member.base.r#expr, session);
            let actual_type = match &member.kind {
                crate::ast::MemberAccessOp::Dot => base_type,
                crate::ast::MemberAccessOp::MinusGreater => {
                    // -> の場合はポインタを逆参照
                    if let Type::Pointer(inner) = base_type {
                        *inner
                    } else {
                        panic!("-> 演算子はポインタ型でのみ使用できます")
                    }
                }
            };

            // 構造体からメンバの型を取得
            match actual_type {
                Type::Struct(s) => s
                    .member
                    .iter()
                    .find(|m| m.ident.name == member.member.name)
                    .map(|m| m.ty.clone())
                    .unwrap_or_else(|| panic!("存在しないメンバ: {}", member.member.name)),
                Type::Union(u) => u
                    .member
                    .iter()
                    .find(|m| m.ident.name == member.member.name)
                    .map(|m| m.ty.clone())
                    .unwrap_or_else(|| panic!("存在しないメンバ: {}", member.member.name)),
                _ => panic!("構造体またはユニオン型でないものにメンバアクセスしています"),
            }
        }
        SemaExpr::Ternary(ternary) => {
            // 三項演算子の型は then_branch と else_branch の共通型
            let then_type = infer_type(&ternary.then_branch.r#expr, session);
            let else_type = infer_type(&ternary.else_branch.r#expr, session);

            // 簡単な共通型推論（実際のCでは複雑なルールがある）
            if then_type == else_type {
                then_type
            } else {
                // 異なる型の場合は促進ルールに従う
                promote_types(&then_type, &else_type)
            }
        }
        SemaExpr::Cast(cast) => {
            // キャストの型は明示的に指定された型
            *cast.r#type.clone()
        }
        SemaExpr::Comma(comma) => {
            // コンマ演算子の型は最後の式の型
            if let Some(last_expr) = comma.assigns.last() {
                infer_type(&last_expr.r#expr, session)
            } else {
                Type::Void
            }
        }
        SemaExpr::Sizeof(_) => Type::Int,
    }
}

fn infer_binary_type(binary: &Binary, session: &mut Session) -> Type {
    let lhs_type = infer_type(&binary.lhs.r#expr, session);
    let rhs_type = infer_type(&binary.rhs.r#expr, session);

    match binary.op {
        BinaryOp::Comparison(_) | BinaryOp::Logical(_) => {
            // 比較・論理演算の結果は int (0 or 1)
            Type::Int
        }
        BinaryOp::Arithmetic(_) => {
            // 算術演算は型促進ルールに従う
            promote_types(&lhs_type, &rhs_type)
        }
    }
}

fn infer_unary_type(unary: &Unary, session: &mut Session) -> Type {
    let operand_type = infer_type(&unary.expr.r#expr, session);

    match unary.op {
        UnaryOp::Bang => Type::Int,     // ! の結果は int (0 or 1)
        UnaryOp::Tilde => operand_type, // ~ は元の型を保持
        UnaryOp::Ampersand => {
            // & はポインタ型を生成
            Type::Pointer(Box::new(operand_type))
        }
        UnaryOp::Asterisk => {
            // * はポインタを逆参照
            if let Type::Pointer(inner) = operand_type {
                *inner
            } else {
                panic!("ポインタでないものを逆参照しています")
            }
        }
    }
}

fn promote_types(lhs: &Type, rhs: &Type) -> Type {
    // 簡単な型促進ルール
    match (lhs, rhs) {
        (Type::Double, _) | (_, Type::Double) => Type::Double,
        (Type::Int, Type::Int) => Type::Int,
        (Type::Int, Type::Char) | (Type::Char, Type::Int) => Type::Int,
        (Type::Char, Type::Char) => Type::Char,
        _ => lhs.clone(), // その他の場合は左の型を採用
    }
}
