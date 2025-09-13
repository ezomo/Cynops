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

pub type TypeResult<T> = Result<T, TypeError>;

pub fn program(program: &Program, session: &mut Session) -> TypeResult<Program> {
    let mut resolved_items = Vec::new();

    for item in &program.items {
        let resolved_item = resolve_toplevel(item, session)?;
        resolved_items.push(resolved_item);
    }

    Ok(Program {
        items: resolved_items,
    })
}

fn resolve_toplevel(toplevel: &TopLevel, session: &mut Session) -> TypeResult<TopLevel> {
    match toplevel {
        TopLevel::FunctionDef(func_def) => Ok(TopLevel::FunctionDef(resolve_function_def(
            func_def, session,
        )?)),
        TopLevel::FunctionProto(func_proto) => Ok(TopLevel::FunctionProto(func_proto.clone())),
        TopLevel::Stmt(stmt) => Ok(TopLevel::Stmt(resolve_stmt(stmt, session)?)),
    }
}

fn resolve_function_def(func_def: &FunctionDef, session: &mut Session) -> TypeResult<FunctionDef> {
    session.current_scope = func_def.sig.scope_ptr.get_scope().unwrap();

    // 関数型を平坦化してから処理
    let flattened_func_type = func_def.sig.ty.flat();
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

    let resolved_body = resolve_block(&func_def.body, session)?;

    Ok(FunctionDef {
        sig: FunctionSig {
            ty: flattened_func_type,
            ident: func_def.sig.ident.clone(),
            scope_ptr: session.current_scope(),
        },
        param_names: func_def.param_names.clone(),
        body: resolved_body,
    })
}

fn resolve_block(block: &Block, session: &mut Session) -> TypeResult<Block> {
    session.current_scope = block.scope_par.get_scope().unwrap();

    let mut resolved_statements = Vec::new();
    for stmt in &block.statements {
        let resolved_stmt = resolve_stmt(stmt, session)?;
        resolved_statements.push(Box::new(resolved_stmt));
    }

    Ok(Block {
        statements: resolved_statements,
        scope_par: session.current_scope(),
    })
}

fn resolve_stmt(stmt: &Stmt, session: &mut Session) -> TypeResult<Stmt> {
    match stmt {
        Stmt::ExprStmt(expr) => Ok(Stmt::expr(resolve_typed_expr(expr, session)?)),
        Stmt::DeclStmt(decl) => Ok(Stmt::decl_stmt(resolve_decl_stmt(decl, session)?)),
        Stmt::Control(control) => Ok(Stmt::control(resolve_control(control, session)?)),
        Stmt::Return(ret) => Ok(Stmt::Return(Return {
            value: match &ret.value {
                Some(v) => Some(Box::new(resolve_typed_expr(v, session)?)),
                None => None,
            },
        })),
        Stmt::Block(block) => Ok(Stmt::Block(resolve_block(block, session)?)),
        Stmt::Label(label) => Ok(Stmt::Label(Label {
            name: label.name.clone(),
            stmt: Box::new(resolve_stmt(&label.stmt, session)?),
        })),
        _ => Ok(stmt.clone()),
    }
}

fn resolve_control(control: &Control, session: &mut Session) -> TypeResult<Control> {
    match control {
        Control::If(if_stmt) => Ok(Control::r#if(
            resolve_typed_expr(&if_stmt.cond, session)?,
            resolve_stmt(&if_stmt.then_branch, session)?,
            match &if_stmt.else_branch {
                Some(e) => Some(resolve_stmt(e, session)?),
                None => None,
            },
        )),
        Control::While(while_stmt) => Ok(Control::r#while(
            resolve_typed_expr(&while_stmt.cond, session)?,
            resolve_stmt(&while_stmt.body, session)?,
        )),
        Control::DoWhile(do_while) => Ok(Control::r#do_while(
            resolve_stmt(&do_while.body, session)?,
            resolve_typed_expr(&do_while.cond, session)?,
        )),
        Control::For(for_stmt) => Ok(Control::r#for(
            match &for_stmt.init {
                Some(i) => Some(resolve_typed_expr(i, session)?),
                None => None,
            },
            match &for_stmt.cond {
                Some(c) => Some(resolve_typed_expr(c, session)?),
                None => None,
            },
            match &for_stmt.step {
                Some(s) => Some(resolve_typed_expr(s, session)?),
                None => None,
            },
            resolve_stmt(&for_stmt.body, session)?,
        )),
        Control::Switch(switch) => {
            let mut cases = Vec::new();
            for c in &switch.cases {
                cases.push(resolve_switch_case(c, session)?);
            }
            Ok(Control::r#switch(
                resolve_typed_expr(&switch.cond, session)?,
                cases,
            ))
        }
    }
}

fn resolve_switch_case(case: &SwitchCase, session: &mut Session) -> TypeResult<SwitchCase> {
    match case {
        SwitchCase::Case(c) => Ok(SwitchCase::Case(Case {
            const_expr: resolve_typed_expr(&c.const_expr, session)?,
            stmts: {
                let mut stmts = Vec::new();
                for s in &c.stmts {
                    stmts.push(Box::new(resolve_stmt(s, session)?));
                }
                stmts
            },
        })),
        SwitchCase::Default(d) => Ok(SwitchCase::Default(DefaultCase {
            stmts: {
                let mut stmts = Vec::new();
                for s in &d.stmts {
                    stmts.push(Box::new(resolve_stmt(s, session)?));
                }
                stmts
            },
        })),
    }
}

fn resolve_decl_stmt(decl: &DeclStmt, session: &mut Session) -> TypeResult<DeclStmt> {
    match decl {
        DeclStmt::InitVec(inits) => {
            let mut resolved_inits = Vec::new();
            for init in inits {
                let resolved_init = resolve_init(init, session)?;
                resolved_inits.push(resolved_init);
            }
            Ok(DeclStmt::InitVec(resolved_inits))
        }
        _ => Ok(decl.clone()),
    }
}

fn resolve_init(init: &Init, session: &mut Session) -> TypeResult<Init> {
    session.current_scope = init.r.sympl.scope.get_scope().unwrap();

    let mut resolved_type = init.r.sympl.get_type().unwrap().flat();

    // 配列の長さ推論
    if let Some(init_data) = &init.l {
        infer_array_length(&mut resolved_type, init_data, session);
    }

    let resolved_member_decl = MemberDecl {
        sympl: init.r.sympl.clone(),
    };

    // 初期化データがある場合、型の互換性をチェック
    if let Some(init_data) = &init.l {
        let init_expr = resolve_init_data(init_data, session)?;
        check_init_compatibility(&resolved_type, &init_expr, session)?;
    }

    // 変数をセッションに平坦化された型で登録
    session.register_symbols(
        resolved_member_decl.sympl.ident.clone(),
        resolved_type.clone(),
    );

    Ok(Init {
        r: resolved_member_decl,
        l: init
            .l
            .as_ref()
            .map(|data| resolve_init_data(data, session))
            .transpose()?,
    })
}

fn check_init_compatibility(
    var_type: &Type,
    init_data: &InitData,
    session: &mut Session,
) -> TypeResult<()> {
    match init_data {
        InitData::Expr(expr) => {
            // 両方の型を平坦化して比較
            let var_type_flat = var_type.flat();
            let expr_type_flat = expr.r#type.flat();

            if var_type_flat != expr_type_flat {
                return Err(TypeError::IncompatibleTypes {
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
                    check_init_compatibility(&array.array_of, compound, session)?;
                }
            }
            // 構造体の場合は省略（実装が複雑になるため）
        }
    }
    Ok(())
}

fn infer_array_length(array_type: &mut Type, init_data: &InitData, session: &mut Session) {
    if let Type::Array(array) = array_type {
        if array.length.is_none() {
            if let InitData::Compound(compounds) = init_data {
                let len_expr = TypedExpr::new(Type::Int, SemaExpr::NumInt(compounds.len()));
                array.length = Some(Box::new(len_expr));
            }
        } else {
            let len_expr = TypedExpr::new(
                Type::Int,
                SemaExpr::NumInt(
                    resolve_typed_expr(array.length.clone().unwrap().as_ref(), session)
                        .unwrap()
                        .eval_const()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                ),
            );
            array.length = Some(Box::new(len_expr));
        }
    }
}

fn resolve_init_data(data: &InitData, session: &mut Session) -> TypeResult<InitData> {
    match data {
        InitData::Expr(expr) => Ok(InitData::Expr(resolve_typed_expr(expr, session)?)),
        InitData::Compound(compounds) => {
            let mut resolved_compounds = Vec::new();
            for c in compounds {
                resolved_compounds.push(resolve_init_data(c, session)?);
            }
            Ok(InitData::Compound(resolved_compounds))
        }
    }
}

pub fn resolve_typed_expr(expr: &TypedExpr, session: &mut Session) -> TypeResult<TypedExpr> {
    let resolved_sema_expr = resolve_sema_expr(&expr.r#expr, session)?;
    let inferred_type = infer_type(&resolved_sema_expr, session)?.flat(); // 推論した型も平坦化

    Ok(TypedExpr {
        r#type: inferred_type,
        r#expr: resolved_sema_expr,
    })
}

fn resolve_sema_expr(expr: &SemaExpr, session: &mut Session) -> TypeResult<SemaExpr> {
    match expr {
        SemaExpr::Assign(assign) => {
            let lhs = resolve_typed_expr(&assign.lhs, session)?;
            let rhs = resolve_typed_expr(&assign.rhs, session)?;

            // 代入の型互換性チェック（平坦化された型で比較）
            let lhs_flat = lhs.r#type.flat();
            let rhs_flat = rhs.r#type.flat();

            if lhs_flat != rhs_flat {
                return Err(TypeError::IncompatibleTypes {
                    expected: lhs_flat,
                    found: rhs_flat,
                    context: "assignment".to_string(),
                });
            }

            Ok(SemaExpr::Assign(Assign {
                op: assign.op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }))
        }
        SemaExpr::Binary(binary) => Ok(SemaExpr::Binary(Binary {
            op: binary.op,
            lhs: Box::new(resolve_typed_expr(&binary.lhs, session)?),
            rhs: Box::new(resolve_typed_expr(&binary.rhs, session)?),
        })),
        SemaExpr::Unary(unary) => Ok(SemaExpr::Unary(Unary {
            op: unary.op,
            expr: Box::new(resolve_typed_expr(&unary.expr, session)?),
        })),
        SemaExpr::Ternary(ternary) => Ok(SemaExpr::Ternary(Ternary {
            cond: Box::new(resolve_typed_expr(&ternary.cond, session)?),
            then_branch: Box::new(resolve_typed_expr(&ternary.then_branch, session)?),
            else_branch: Box::new(resolve_typed_expr(&ternary.else_branch, session)?),
        })),
        SemaExpr::Call(call) => {
            let resolved_func = resolve_typed_expr(&call.func, session)?;
            let mut resolved_args = Vec::new();

            for arg in &call.args {
                resolved_args.push(Box::new(resolve_typed_expr(arg, session)?));
            }

            // 関数呼び出しの型チェック
            check_function_call(&resolved_func, &resolved_args, session)?;

            Ok(SemaExpr::Call(Call {
                func: Box::new(resolved_func),
                args: resolved_args,
            }))
        }
        SemaExpr::Subscript(subscript) => Ok(SemaExpr::Subscript(Subscript {
            subject: Box::new(resolve_typed_expr(&subscript.subject, session)?),
            index: Box::new(resolve_typed_expr(&subscript.index, session)?),
        })),
        SemaExpr::MemberAccess(member) => Ok(SemaExpr::MemberAccess(MemberAccess {
            base: Box::new(resolve_typed_expr(&member.base, session)?),
            member: member.member.clone(),
            kind: member.kind.clone(),
        })),
        SemaExpr::Cast(cast) => {
            // キャスト型も平坦化
            let flattened_cast_type = cast.type_to.flat();
            let expr = resolve_typed_expr(&cast.expr, session)?;
            Ok(SemaExpr::cast(
                flattened_cast_type,
                expr.r#type.flat(),
                resolve_typed_expr(&cast.expr, session)?,
            ))
        }
        SemaExpr::Comma(comma) => Ok(SemaExpr::Comma(Comma {
            assigns: {
                let mut assigns = Vec::new();
                for a in &comma.assigns {
                    assigns.push(resolve_typed_expr(a, session)?);
                }
                assigns
            },
        })),
        SemaExpr::Sizeof(sizeof) => {
            let resolved_sizeof = match sizeof {
                Sizeof::Type(ty) => Sizeof::Type(ty.flat()), // sizeof内の型も平坦化
                Sizeof::TypedExpr(expr) => {
                    Sizeof::TypedExpr(Box::new(resolve_typed_expr(expr, session)?))
                }
            };
            Ok(SemaExpr::Sizeof(resolved_sizeof))
        }
        _ => Ok(expr.clone()),
    }
}

fn check_function_call(
    func_expr: &TypedExpr,
    args: &[Box<TypedExpr>],
    _session: &mut Session,
) -> TypeResult<()> {
    // 関数型も平坦化してチェック
    let func_type_flat = func_expr.r#type.flat();

    if let Type::Func(func) = func_type_flat {
        // 引数の数をチェック
        if func.params.last().unwrap() != &Type::Void
            && func.params.last().unwrap() != &Type::DotDotDot
            && func.params.len() != args.len()
        {
            return Err(TypeError::IncompatibleTypes {
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

            if expected_flat != actual_flat {
                return Err(TypeError::IncompatibleTypes {
                    expected: expected_flat,
                    found: actual_flat,
                    context: format!("function call argument {} ({})", i + 1, func_expr.oneline()),
                });
            }
        }

        Ok(())
    } else {
        // 関数型でない場合のエラー
        Err(TypeError::InvalidOperation {
            op: "function call".to_string(),
            operand_type: func_type_flat,
        })
    }
}

fn infer_type(expr: &SemaExpr, session: &mut Session) -> TypeResult<Type> {
    match expr {
        SemaExpr::NumInt(_) => Ok(Type::Int),
        SemaExpr::NumFloat(_) => Ok(Type::Double),
        SemaExpr::Char(_) => Ok(Type::Char),
        SemaExpr::String(this) => Ok(Type::Array(Array {
            array_of: Box::new(Type::Char),
            length: Some(Box::new(TypedExpr::new(
                Type::Int,
                SemaExpr::NumInt(this.len()),
            ))),
        })),
        SemaExpr::Symbol(symbol) => session
            .get_type(&symbol.ident)
            .map(|t| t.flat()) // シンボルの型も平坦化
            .ok_or_else(|| TypeError::UndefinedVariable(symbol.ident.name.clone())),
        SemaExpr::Binary(binary) => infer_binary_type(binary, session),
        SemaExpr::Unary(unary) => infer_unary_type(unary, session),
        SemaExpr::Assign(assign) => Ok(infer_type(&assign.lhs.r#expr, session)?),
        SemaExpr::Call(call) => {
            let func_type = infer_type(&call.func.r#expr, session)?.flat();
            if let Type::Func(func) = func_type {
                Ok((*func.return_type).flat()) // 戻り値型も平坦化
            } else {
                Err(TypeError::InvalidOperation {
                    op: "function call".to_string(),
                    operand_type: func_type,
                })
            }
        }
        SemaExpr::Subscript(subscript) => {
            let subject_type = infer_type(&subscript.subject.r#expr, session)?.flat();
            match subject_type {
                Type::Array(array) => Ok((*array.array_of).flat()),
                Type::Pointer(inner) => Ok((*inner).flat()),
                _ => Err(TypeError::InvalidOperation {
                    op: "array subscript".to_string(),
                    operand_type: subject_type,
                }),
            }
        }
        SemaExpr::MemberAccess(member) => {
            let base_type = infer_type(&member.base.r#expr, session)?.flat();
            let actual_type = match &member.kind {
                MemberAccessOp::Dot => base_type.clone(),
                _ => unreachable!(),
            };

            match actual_type.flat() {
                Type::Struct(ref s) => s
                    .member
                    .iter()
                    .find(|m| m.sympl.ident.name == member.member.name)
                    .map(|m| m.sympl.get_type().unwrap().flat()) // メンバー型も平坦化
                    .ok_or_else(|| TypeError::InvalidMemberAccess {
                        base_type: actual_type.clone(),
                        member: member.member.name.clone(),
                    }),
                Type::Union(u) => u
                    .member
                    .iter()
                    .find(|m| m.sympl.ident.name == member.member.name)
                    .map(|m| m.sympl.get_type().unwrap().flat()) // メンバー型も平坦化
                    .ok_or_else(|| TypeError::InvalidMemberAccess {
                        base_type: actual_type.clone(),
                        member: member.member.name.clone(),
                    }),
                _ => Err(TypeError::InvalidMemberAccess {
                    base_type: actual_type,
                    member: member.member.name.clone(),
                }),
            }
        }
        SemaExpr::Ternary(ternary) => {
            let then_type = infer_type(&ternary.then_branch.r#expr, session)?.flat();
            let else_type = infer_type(&ternary.else_branch.r#expr, session)?.flat();

            // 三項演算子でも完全一致を要求（平坦化後で比較）
            if then_type == else_type {
                Ok(then_type)
            } else {
                // 型が一致しない場合はエラー
                Err(TypeError::IncompatibleTypes {
                    expected: then_type,
                    found: else_type,
                    context: "ternary operator branches".to_string(),
                })
            }
        }
        SemaExpr::Cast(cast) => Ok(cast.type_to.flat()), // キャストの結果型も平坦化
        SemaExpr::Comma(comma) => {
            if let Some(last_expr) = comma.assigns.last() {
                infer_type(&last_expr.r#expr, session)
            } else {
                Ok(Type::Void)
            }
        }
        SemaExpr::Sizeof(_) => Ok(Type::Int),
    }
}

fn infer_binary_type(binary: &Binary, session: &mut Session) -> TypeResult<Type> {
    let lhs_type = infer_type(&binary.lhs.r#expr, session)?.flat();
    let rhs_type = infer_type(&binary.rhs.r#expr, session)?.flat();

    match binary.op {
        BinaryOp::Comparison(_) | BinaryOp::Logical(_) => {
            // 比較・論理演算は結果がint型
            Ok(Type::Int)
        }
        BinaryOp::Arithmetic(_) => {
            // 算術演算では両オペランドの型が一致している必要がある（平坦化後で比較）
            if lhs_type == rhs_type {
                Ok(lhs_type)
            } else {
                Err(TypeError::IncompatibleTypes {
                    expected: lhs_type,
                    found: rhs_type,
                    context: "arithmetic operation".to_string(),
                })
            }
        }
    }
}

fn infer_unary_type(unary: &Unary, session: &mut Session) -> TypeResult<Type> {
    let operand_type = infer_type(&unary.expr.r#expr, session)?.flat();

    match unary.op {
        UnaryOp::Bang => Ok(Type::Int),
        UnaryOp::Tilde => Ok(operand_type),
        UnaryOp::Ampersand => Ok(Type::Pointer(Box::new(operand_type))),
        UnaryOp::Asterisk => {
            if let Type::Pointer(inner) = operand_type {
                Ok((*inner).flat()) // デリファレンス結果も平坦化
            } else {
                Err(TypeError::InvalidOperation {
                    op: "dereference".to_string(),
                    operand_type,
                })
            }
        }
        _ => Ok(operand_type), // その他の単項演算子
    }
}
