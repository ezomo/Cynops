use super::ast::*;
use crate::op::*;
use crate::visualize::*;

#[derive(Debug, Clone)]
pub enum TypeError {
    IncompatibleTypes {
        expected: Type,
        found: Type,
        context: String,
    },
    UndefinedVariable(String),
    InvalidOperation {
        op: String,
        operand_type: Type,
    },
    InvalidMemberAccess {
        base_type: Type,
        member: String,
    },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::IncompatibleTypes {
                expected,
                found,
                context,
            } => {
                write!(
                    f,
                    "Type error in {}: expected '{}', found '{}'",
                    context,
                    expected.to_rust_format(),
                    found.to_rust_format()
                )
            }
            TypeError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            TypeError::InvalidOperation { op, operand_type } => {
                write!(
                    f,
                    "Invalid operation '{}' for type '{}'",
                    op,
                    operand_type.to_rust_format()
                )
            }
            TypeError::InvalidMemberAccess { base_type, member } => {
                write!(
                    f,
                    "Invalid member access '{}' for type '{}'",
                    member,
                    base_type.to_rust_format()
                )
            }
        }
    }
}

// 型チェック結果を格納する構造体（エラーと結果の両方を保持）
#[derive(Debug)]
pub struct TypeCheckResult<T> {
    pub result: T,
    pub errors: Vec<TypeError>,
}

impl<T> TypeCheckResult<T> {
    fn new(result: T) -> Self {
        Self {
            result,
            errors: Vec::new(),
        }
    }
}

// Error型かどうかをチェックするヘルパー関数
fn is_error_type(ty: &Type) -> bool {
    matches!(ty, Type::Error)
}

// Error型を含む場合はError型を返すヘルパー関数
fn propagate_error_type(types: &[&Type]) -> Option<Type> {
    if types.iter().any(|t| is_error_type(t)) {
        Some(Type::Error)
    } else {
        None
    }
}

pub fn program(program: &Program, session: &mut Session) -> TypeCheckResult<Program> {
    let mut resolved_items = Vec::new();
    let mut all_errors = Vec::new();

    for item in &program.items {
        let mut result = resolve_toplevel(item, session);
        all_errors.append(&mut result.errors);
        resolved_items.push(result.result);
    }

    TypeCheckResult {
        result: Program {
            items: resolved_items,
        },
        errors: all_errors,
    }
}

fn resolve_toplevel(toplevel: &TopLevel, session: &mut Session) -> TypeCheckResult<TopLevel> {
    match toplevel {
        TopLevel::FunctionDef(func_def) => {
            let result = resolve_function_def(func_def, session);
            TypeCheckResult {
                result: TopLevel::FunctionDef(result.result),
                errors: result.errors,
            }
        }
        TopLevel::FunctionProto(func_proto) => {
            TypeCheckResult::new(TopLevel::FunctionProto(func_proto.clone()))
        }
        TopLevel::Stmt(stmt) => {
            let result = resolve_stmt(stmt, session);
            TypeCheckResult {
                result: TopLevel::Stmt(result.result),
                errors: result.errors,
            }
        }
    }
}

// TODO カス　絶対になおす
fn resolve_function_def(
    func_def: &FunctionDef,
    session: &mut Session,
) -> TypeCheckResult<FunctionDef> {
    let mut errors = Vec::new();

    session.current_scope = func_def.sig.symbol.scope.get_scope().unwrap();

    // 関数型を平坦化してから処理
    let flattened_func_type = func_def.sig.symbol.get_type().unwrap().flat();
    if let Some(func_type) = flattened_func_type.as_func() {
        for (param_name, param_type) in func_def
            .param_names
            .clone()
            .iter_mut()
            .zip(&func_type.params)
        {
            // パラメータ型も平坦化して登録
            param_name
                .scope
                .register_symbols(param_name.ident.clone(), param_type.flat());
        }
    }

    let mut body_result = resolve_block(&func_def.body, session);
    errors.append(&mut body_result.errors);

    TypeCheckResult {
        result: FunctionDef {
            sig: FunctionSig {
                symbol: Symbol::new(func_def.sig.symbol.ident.clone(), session.current_scope()),
            },
            param_names: func_def.param_names.clone(),
            body: body_result.result,
        },
        errors,
    }
}

fn resolve_block(block: &Block, session: &mut Session) -> TypeCheckResult<Block> {
    let mut errors = Vec::new();

    session.current_scope = block.scope_par.get_scope().unwrap();

    let mut resolved_statements = Vec::new();
    for stmt in &block.statements {
        let mut stmt_result = resolve_stmt(stmt, session);
        errors.append(&mut stmt_result.errors);
        resolved_statements.push(Box::new(stmt_result.result));
    }

    TypeCheckResult {
        result: Block {
            statements: resolved_statements,
            scope_par: session.current_scope(),
        },
        errors,
    }
}

fn resolve_stmt(stmt: &Stmt, session: &mut Session) -> TypeCheckResult<Stmt> {
    let mut errors = Vec::new();

    let result = match stmt {
        Stmt::ExprStmt(expr) => {
            let mut expr_result = resolve_typed_expr(expr, session);
            errors.append(&mut expr_result.errors);
            Stmt::expr(expr_result.result)
        }
        Stmt::DeclStmt(decl) => {
            let mut decl_result = resolve_decl_stmt(decl, session);
            errors.append(&mut decl_result.errors);
            Stmt::decl_stmt(decl_result.result)
        }
        Stmt::Control(control) => {
            let mut control_result = resolve_control(control, session);
            errors.append(&mut control_result.errors);
            Stmt::control(control_result.result)
        }
        Stmt::Return(ret) => Stmt::Return(Return {
            value: match &ret.value {
                Some(v) => {
                    let mut v_result = resolve_typed_expr(v, session);
                    errors.append(&mut v_result.errors);
                    Some(Box::new(v_result.result))
                }
                None => None,
            },
        }),
        Stmt::Block(block) => {
            let mut block_result = resolve_block(block, session);
            errors.append(&mut block_result.errors);
            Stmt::Block(block_result.result)
        }
        Stmt::Label(label) => {
            let mut stmt_result = resolve_stmt(&label.stmt, session);
            errors.append(&mut stmt_result.errors);
            Stmt::Label(Label {
                name: label.name.clone(),
                stmt: Box::new(stmt_result.result),
            })
        }
        _ => stmt.clone(),
    };

    TypeCheckResult { result, errors }
}

fn resolve_control(control: &Control, session: &mut Session) -> TypeCheckResult<Control> {
    let mut errors = Vec::new();

    let result = match control {
        Control::If(if_stmt) => {
            let mut cond_result = resolve_typed_expr(&if_stmt.cond, session);
            let mut then_result = resolve_stmt(&if_stmt.then_branch, session);
            errors.append(&mut cond_result.errors);
            errors.append(&mut then_result.errors);

            let else_branch = match &if_stmt.else_branch {
                Some(e) => {
                    let mut else_result = resolve_stmt(e, session);
                    errors.append(&mut else_result.errors);
                    Some(else_result.result)
                }
                None => None,
            };

            Control::r#if(cond_result.result, then_result.result, else_branch)
        }
        Control::While(while_stmt) => {
            let mut cond_result = resolve_typed_expr(&while_stmt.cond, session);
            let mut body_result = resolve_stmt(&while_stmt.body, session);
            errors.append(&mut cond_result.errors);
            errors.append(&mut body_result.errors);

            Control::r#while(cond_result.result, body_result.result)
        }
        Control::DoWhile(do_while) => {
            let mut body_result = resolve_stmt(&do_while.body, session);
            let mut cond_result = resolve_typed_expr(&do_while.cond, session);
            errors.append(&mut body_result.errors);
            errors.append(&mut cond_result.errors);

            Control::r#do_while(body_result.result, cond_result.result)
        }
        Control::For(for_stmt) => {
            let init = match &for_stmt.init {
                Some(i) => {
                    let mut init_result = resolve_typed_expr(i, session);
                    errors.append(&mut init_result.errors);
                    Some(init_result.result)
                }
                None => None,
            };
            let cond = match &for_stmt.cond {
                Some(c) => {
                    let mut cond_result = resolve_typed_expr(c, session);
                    errors.append(&mut cond_result.errors);
                    Some(cond_result.result)
                }
                None => None,
            };
            let step = match &for_stmt.step {
                Some(s) => {
                    let mut step_result = resolve_typed_expr(s, session);
                    errors.append(&mut step_result.errors);
                    Some(step_result.result)
                }
                None => None,
            };
            let mut body_result = resolve_stmt(&for_stmt.body, session);
            errors.append(&mut body_result.errors);

            Control::r#for(init, cond, step, body_result.result)
        }
        Control::Switch(switch) => {
            let mut cond_result = resolve_typed_expr(&switch.cond, session);
            errors.append(&mut cond_result.errors);

            let mut cases = Vec::new();
            for c in &switch.cases {
                let mut case_result = resolve_switch_case(c, session);
                errors.append(&mut case_result.errors);
                cases.push(case_result.result);
            }
            Control::r#switch(cond_result.result, cases)
        }
    };

    TypeCheckResult { result, errors }
}

fn resolve_switch_case(case: &SwitchCase, session: &mut Session) -> TypeCheckResult<SwitchCase> {
    let mut errors = Vec::new();

    let result = match case {
        SwitchCase::Case(c) => {
            let mut const_result = resolve_typed_expr(&c.const_expr, session);
            errors.append(&mut const_result.errors);

            let mut stmts = Vec::new();
            for s in &c.stmts {
                let mut stmt_result = resolve_stmt(s, session);
                errors.append(&mut stmt_result.errors);
                stmts.push(Box::new(stmt_result.result));
            }

            SwitchCase::Case(Case {
                const_expr: const_result.result,
                stmts,
            })
        }
        SwitchCase::Default(d) => {
            let mut stmts = Vec::new();
            for s in &d.stmts {
                let mut stmt_result = resolve_stmt(s, session);
                errors.append(&mut stmt_result.errors);
                stmts.push(Box::new(stmt_result.result));
            }

            SwitchCase::Default(DefaultCase { stmts })
        }
    };

    TypeCheckResult { result, errors }
}

fn resolve_decl_stmt(decl: &DeclStmt, session: &mut Session) -> TypeCheckResult<DeclStmt> {
    let mut errors = Vec::new();

    let result = match decl {
        DeclStmt::InitVec(inits) => {
            let mut resolved_inits = Vec::new();
            for init in inits {
                let mut init_result = resolve_init(init, session);
                errors.append(&mut init_result.errors);
                resolved_inits.push(init_result.result);
            }
            DeclStmt::InitVec(resolved_inits)
        }
        _ => decl.clone(),
    };

    TypeCheckResult { result, errors }
}

fn resolve_init(init: &Init, session: &mut Session) -> TypeCheckResult<Init> {
    let mut errors = Vec::new();

    session.current_scope = init.l.scope.get_scope().unwrap();

    let mut resolved_type = init.l.get_type().unwrap().flat();

    // 配列の長さ推論
    if let Some(init_data) = &init.r {
        infer_array_length(&mut resolved_type, init_data, session);
    }

    let resolved_member_decl = init.l.clone();

    // 初期化データがある場合、型の互換性をチェック
    let init_data_result = if let Some(init_data) = &init.r {
        let mut init_result = resolve_init_data(init_data, session);
        errors.append(&mut init_result.errors);

        // 型の互換性チェック
        check_init_compatibility(&resolved_type, &init_result.result, session, &mut errors);

        Some(init_result.result)
    } else {
        None
    };

    // 変数をセッションに平坦化された型で登録
    session.register_symbols(resolved_member_decl.ident.clone(), resolved_type.clone());

    TypeCheckResult {
        result: Init {
            l: resolved_member_decl,
            r: init_data_result,
        },
        errors,
    }
}

fn check_init_compatibility(
    var_type: &Type,
    init_data: &InitData,
    session: &mut Session,
    errors: &mut Vec<TypeError>,
) {
    match init_data {
        InitData::Expr(expr) => {
            // 両方の型を平坦化して比較
            let var_type_flat = var_type.flat();
            let expr_type_flat = expr.r#type.flat();

            // Error型の場合は互換性チェックをスキップ
            if !is_error_type(&var_type_flat)
                && !is_error_type(&expr_type_flat)
                && var_type_flat != expr_type_flat
            {
                errors.push(TypeError::IncompatibleTypes {
                    expected: var_type_flat,
                    found: expr_type_flat,
                    context: "variable initialization".to_string(),
                });
            }
        }
        InitData::Compound(compounds) => {
            // 配列や構造体の複合初期化子の場合
            let flat_var_type = var_type.flat();
            if let Type::Array(array) = flat_var_type {
                for compound in compounds {
                    check_init_compatibility(&array.array_of, compound, session, errors);
                }
            }
            // 構造体の場合は省略（実装が複雑になるため）
        }
    }
}

fn infer_array_length(array_type: &mut Type, init_data: &InitData, session: &mut Session) {
    if let Type::Array(array) = array_type {
        if array.length.is_none() {
            if let InitData::Compound(compounds) = init_data {
                let len_expr = TypedExpr::new(Type::Int, SemaExpr::NumInt(compounds.len()));
                array.length = Some(Box::new(len_expr));
            }

            if let InitData::Expr(this) = init_data {
                if let SemaExpr::String(s) = &this.r#expr {
                    let len_expr = TypedExpr::new(Type::Int, SemaExpr::NumInt(s.len()));
                    array.length = Some(Box::new(len_expr));
                }
            }
        } else {
            // エラーハンドリングを改善
            let resolved = resolve_typed_expr(array.length.clone().unwrap().as_ref(), session);
            if let Ok(const_val) = resolved.result.eval_const() {
                if let Ok(len_val) = const_val.try_into() {
                    let len_expr = TypedExpr::new(Type::Int, SemaExpr::NumInt(len_val));
                    array.length = Some(Box::new(len_expr));
                }
            }
        }
    }
}

fn resolve_init_data(data: &InitData, session: &mut Session) -> TypeCheckResult<InitData> {
    let mut errors = Vec::new();

    let result = match data {
        InitData::Expr(expr) => {
            let mut expr_result = resolve_typed_expr(expr, session);
            errors.append(&mut expr_result.errors);
            InitData::Expr(expr_result.result)
        }
        InitData::Compound(compounds) => {
            let mut resolved_compounds = Vec::new();
            for c in compounds {
                let mut compound_result = resolve_init_data(c, session);
                errors.append(&mut compound_result.errors);
                resolved_compounds.push(compound_result.result);
            }
            InitData::Compound(resolved_compounds)
        }
    };

    TypeCheckResult { result, errors }
}

pub fn resolve_typed_expr(expr: &TypedExpr, session: &mut Session) -> TypeCheckResult<TypedExpr> {
    let sema_result = resolve_sema_expr(&expr.r#expr, session);
    let mut errors = sema_result.errors;

    let inferred_type = infer_type(&sema_result.result, session, &mut errors).flat();

    TypeCheckResult {
        result: TypedExpr {
            r#type: inferred_type,
            r#expr: sema_result.result,
        },
        errors,
    }
}

fn resolve_sema_expr(expr: &SemaExpr, session: &mut Session) -> TypeCheckResult<SemaExpr> {
    let mut errors = Vec::new();

    let result = match expr {
        SemaExpr::Assign(assign) => {
            let mut lhs_result = resolve_typed_expr(&assign.lhs, session);
            let mut rhs_result = resolve_typed_expr(&assign.rhs, session);
            errors.append(&mut lhs_result.errors);
            errors.append(&mut rhs_result.errors);

            // 代入の型互換性チェック（平坦化された型で比較）
            let lhs_flat = lhs_result.result.r#type.flat();
            let rhs_flat = rhs_result.result.r#type.flat();

            // Error型の場合は互換性チェックをスキップ、そうでなければエラーをログして継続
            if !is_error_type(&lhs_flat) && !is_error_type(&rhs_flat) && lhs_flat != rhs_flat {
                errors.push(TypeError::IncompatibleTypes {
                    expected: lhs_flat,
                    found: rhs_flat,
                    context: "assignment".to_string(),
                });
            }

            SemaExpr::Assign(Assign {
                op: assign.op,
                lhs: Box::new(lhs_result.result),
                rhs: Box::new(rhs_result.result),
            })
        }
        SemaExpr::Binary(binary) => {
            let mut lhs_result = resolve_typed_expr(&binary.lhs, session);
            let mut rhs_result = resolve_typed_expr(&binary.rhs, session);
            errors.append(&mut lhs_result.errors);
            errors.append(&mut rhs_result.errors);

            SemaExpr::Binary(Binary {
                op: binary.op,
                lhs: Box::new(lhs_result.result),
                rhs: Box::new(rhs_result.result),
            })
        }
        SemaExpr::Unary(unary) => {
            let mut expr_result = resolve_typed_expr(&unary.expr, session);
            errors.append(&mut expr_result.errors);

            SemaExpr::Unary(Unary {
                op: unary.op,
                expr: Box::new(expr_result.result),
            })
        }
        SemaExpr::Ternary(ternary) => {
            let mut cond_result = resolve_typed_expr(&ternary.cond, session);
            let mut then_result = resolve_typed_expr(&ternary.then_branch, session);
            let mut else_result = resolve_typed_expr(&ternary.else_branch, session);
            errors.append(&mut cond_result.errors);
            errors.append(&mut then_result.errors);
            errors.append(&mut else_result.errors);

            SemaExpr::Ternary(Ternary {
                cond: Box::new(cond_result.result),
                then_branch: Box::new(then_result.result),
                else_branch: Box::new(else_result.result),
            })
        }
        SemaExpr::Call(call) => {
            let mut func_result = resolve_typed_expr(&call.func, session);
            errors.append(&mut func_result.errors);

            let mut resolved_args = Vec::new();
            for arg in &call.args {
                let mut arg_result = resolve_typed_expr(arg, session);
                errors.append(&mut arg_result.errors);
                resolved_args.push(Box::new(arg_result.result));
            }

            // 関数呼び出しの型チェック
            check_function_call(&func_result.result, &resolved_args, session, &mut errors);

            SemaExpr::Call(Call {
                func: Box::new(func_result.result),
                args: resolved_args,
            })
        }
        SemaExpr::Subscript(subscript) => {
            let mut subject_result = resolve_typed_expr(&subscript.subject, session);
            let mut index_result = resolve_typed_expr(&subscript.index, session);
            errors.append(&mut subject_result.errors);
            errors.append(&mut index_result.errors);

            SemaExpr::Subscript(Subscript {
                subject: Box::new(subject_result.result),
                index: Box::new(index_result.result),
            })
        }
        SemaExpr::MemberAccess(member) => {
            let mut base_result = resolve_typed_expr(&member.base, session);
            errors.append(&mut base_result.errors);

            SemaExpr::MemberAccess(MemberAccess {
                base: Box::new(base_result.result),
                member: member.member.clone(),
                kind: member.kind.clone(),
            })
        }
        SemaExpr::Cast(cast) => {
            let flattened_cast_type = cast.type_to.flat();
            let mut expr_result = resolve_typed_expr(&cast.expr, session);
            errors.append(&mut expr_result.errors);

            SemaExpr::cast(
                flattened_cast_type,
                expr_result.result.r#type.flat(),
                expr_result.result,
            )
        }
        SemaExpr::Comma(comma) => {
            let mut assigns = Vec::new();
            for a in &comma.assigns {
                let mut assign_result = resolve_typed_expr(a, session);
                errors.append(&mut assign_result.errors);
                assigns.push(assign_result.result);
            }

            SemaExpr::Comma(Comma { assigns })
        }
        SemaExpr::Sizeof(sizeof) => {
            let resolved_sizeof = match sizeof {
                Sizeof::Type(ty) => Sizeof::Type(ty.flat()), // sizeof内の型も平坦化
                Sizeof::TypedExpr(expr) => {
                    let mut expr_result = resolve_typed_expr(expr, session);
                    errors.append(&mut expr_result.errors);
                    Sizeof::TypedExpr(Box::new(expr_result.result))
                }
            };
            SemaExpr::Sizeof(resolved_sizeof)
        }
        _ => expr.clone(),
    };

    TypeCheckResult { result, errors }
}

fn check_function_call(
    func_expr: &TypedExpr,
    args: &[Box<TypedExpr>],
    _session: &mut Session,
    errors: &mut Vec<TypeError>,
) {
    // 関数型も平坦化してチェック
    let func_type_flat = func_expr.r#type.flat();

    // Error型の場合はチェックをスキップ
    if is_error_type(&func_type_flat) {
        return;
    }

    if let Type::Func(func) = func_type_flat {
        // 引数の数をチェック
        if func.params.last().unwrap() != &Type::Void
            && func.params.last().unwrap() != &Type::DotDotDot
            && func.params.len() != args.len()
        {
            errors.push(TypeError::IncompatibleTypes {
                expected: Type::Func(func.clone()),
                found: Type::Func(Func {
                    return_type: func.return_type.clone(),
                    params: args.iter().map(|arg| arg.r#type.clone()).collect(),
                }),
                context: format!(
                    "function call argument count: expected {}, found {}",
                    func.params.len(),
                    args.len()
                ),
            });
        }

        // 各引数の型をチェック（平坦化して比較）
        for (i, (expected_param, actual_arg)) in func.params.iter().zip(args.iter()).enumerate() {
            if expected_param == &Type::DotDotDot {
                break;
            }
            let expected_flat = expected_param.flat();
            let actual_flat = actual_arg.r#type.flat();

            // Error型の場合は型チェックをスキップ
            if !is_error_type(&expected_flat)
                && !is_error_type(&actual_flat)
                && expected_flat != actual_flat
            {
                errors.push(TypeError::IncompatibleTypes {
                    expected: expected_flat,
                    found: actual_flat,
                    context: format!("function call argument {} ({})", i + 1, func_expr.oneline()),
                });
            }
        }
    } else {
        // 関数型でない場合のエラー
        errors.push(TypeError::InvalidOperation {
            op: "function call".to_string(),
            operand_type: func_type_flat,
        });
    }
}

fn infer_type(expr: &SemaExpr, session: &mut Session, errors: &mut Vec<TypeError>) -> Type {
    match expr {
        SemaExpr::NumInt(_) => Type::Int,
        SemaExpr::NumFloat(_) => Type::Double,
        SemaExpr::Char(_) => Type::Char,
        SemaExpr::String(this) => Type::Array(Array {
            array_of: Box::new(Type::Char),
            length: Some(Box::new(TypedExpr::new(
                Type::Int,
                SemaExpr::NumInt(this.len()),
            ))),
        }),
        SemaExpr::Symbol(symbol) => {
            match session.get_type(&symbol.ident) {
                Some(t) => t.flat(), // シンボルの型も平坦化
                None => {
                    errors.push(TypeError::UndefinedVariable(symbol.ident.name.clone()));
                    Type::Error // 未定義変数の場合はError型を返す
                }
            }
        }
        SemaExpr::Binary(binary) => infer_binary_type(binary, session, errors),
        SemaExpr::Unary(unary) => infer_unary_type(unary, session, errors),
        SemaExpr::Assign(assign) => infer_type(&assign.lhs.r#expr, session, errors),
        SemaExpr::Call(call) => {
            let func_type = infer_type(&call.func.r#expr, session, errors).flat();

            // Error型の場合はError型を返す
            if is_error_type(&func_type) {
                return Type::Error;
            }

            if let Type::Func(func) = func_type {
                (*func.return_type).flat() // 戻り値型も平坦化
            } else {
                errors.push(TypeError::InvalidOperation {
                    op: "function call".to_string(),
                    operand_type: func_type,
                });
                Type::Error
            }
        }
        SemaExpr::Subscript(subscript) => {
            let subject_type = infer_type(&subscript.subject.r#expr, session, errors).flat();

            // Error型の場合はError型を返す
            if is_error_type(&subject_type) {
                return Type::Error;
            }

            match subject_type {
                Type::Array(array) => (*array.array_of).flat(),
                Type::Pointer(inner) => (*inner).flat(),
                _ => {
                    errors.push(TypeError::InvalidOperation {
                        op: "array subscript".to_string(),
                        operand_type: subject_type,
                    });
                    Type::Error
                }
            }
        }
        SemaExpr::MemberAccess(member) => {
            let base_type = infer_type(&member.base.r#expr, session, errors).flat();

            // Error型の場合はError型を返す
            if is_error_type(&base_type) {
                return Type::Error;
            }

            let actual_type = match &member.kind {
                MemberAccessOp::Dot => base_type.clone(),
                _ => unreachable!(),
            };

            match actual_type.flat() {
                Type::Struct(ref s) => {
                    match s.member
                        .iter()
                        .find(|m| m.ident.name == member.member.name)
                        .map(|m| m.get_type().unwrap().flat()) // メンバー型も平坦化
                    {
                        Some(member_type) => member_type,
                        None => {
                            errors.push(TypeError::InvalidMemberAccess {
                                base_type: actual_type.clone(),
                                member: member.member.name.clone(),
                            });
                            Type::Error
                        }
                    }
                }
                Type::Union(u) => {
                    match u.member
                        .iter()
                        .find(|m| m.ident.name == member.member.name)
                        .map(|m| m.get_type().unwrap().flat()) // メンバー型も平坦化
                    {
                        Some(member_type) => member_type,
                        None => {
                            errors.push(TypeError::InvalidMemberAccess {
                                base_type: actual_type.clone(),
                                member: member.member.name.clone(),
                            });
                            Type::Error
                        }
                    }
                }
                _ => {
                    errors.push(TypeError::InvalidMemberAccess {
                        base_type: actual_type,
                        member: member.member.name.clone(),
                    });
                    Type::Error
                }
            }
        }
        SemaExpr::Ternary(ternary) => {
            let then_type = infer_type(&ternary.then_branch.r#expr, session, errors).flat();
            let else_type = infer_type(&ternary.else_branch.r#expr, session, errors).flat();

            // Error型がある場合はError型を伝播
            if let Some(error_type) = propagate_error_type(&[&then_type, &else_type]) {
                return error_type;
            }

            // 三項演算子でも完全一致を要求（平坦化後で比較）
            if then_type == else_type {
                then_type
            } else {
                // 型が一致しない場合はエラーを記録してError型を返す
                errors.push(TypeError::IncompatibleTypes {
                    expected: then_type,
                    found: else_type,
                    context: "ternary operator branches".to_string(),
                });
                Type::Error
            }
        }
        SemaExpr::Cast(cast) => cast.type_to.flat(), // キャストの結果型も平坦化
        SemaExpr::Comma(comma) => {
            if let Some(last_expr) = comma.assigns.last() {
                infer_type(&last_expr.r#expr, session, errors)
            } else {
                Type::Void
            }
        }
        SemaExpr::Sizeof(_) => Type::Int,
    }
}

fn infer_binary_type(binary: &Binary, session: &mut Session, errors: &mut Vec<TypeError>) -> Type {
    let lhs_type = infer_type(&binary.lhs.r#expr, session, errors).flat();
    let rhs_type = infer_type(&binary.rhs.r#expr, session, errors).flat();

    // Error型がある場合はError型を伝播
    if let Some(error_type) = propagate_error_type(&[&lhs_type, &rhs_type]) {
        return error_type;
    }

    match binary.op {
        BinaryOp::Comparison(_) | BinaryOp::Logical(_) => {
            // 比較・論理演算は結果がint型
            Type::Int
        }
        BinaryOp::Arithmetic(_) => {
            // 算術演算では両オペランドの型が一致している必要がある（平坦化後で比較）
            if lhs_type == rhs_type {
                lhs_type
            } else {
                errors.push(TypeError::IncompatibleTypes {
                    expected: lhs_type,
                    found: rhs_type,
                    context: "arithmetic operation".to_string(),
                });
                Type::Error
            }
        }
    }
}

fn infer_unary_type(unary: &Unary, session: &mut Session, errors: &mut Vec<TypeError>) -> Type {
    let operand_type = infer_type(&unary.expr.r#expr, session, errors).flat();

    // Error型の場合はError型を返す
    if is_error_type(&operand_type) {
        return Type::Error;
    }

    match unary.op {
        UnaryOp::Bang => Type::Int,
        UnaryOp::Tilde => operand_type,
        UnaryOp::Ampersand => Type::Pointer(Box::new(operand_type)),
        UnaryOp::Asterisk => {
            if let Type::Pointer(inner) = operand_type {
                (*inner).flat() // デリファレンス結果も平坦化
            } else {
                errors.push(TypeError::InvalidOperation {
                    op: "dereference".to_string(),
                    operand_type,
                });
                Type::Error
            }
        }
        _ => operand_type, // その他の単項演算子
    }
}
