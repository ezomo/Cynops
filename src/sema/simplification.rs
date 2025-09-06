use crate::ast::*;
use crate::op::*;

#[derive(Debug)]
pub struct Session {
    id: usize,
}

impl Session {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn id(&mut self) -> usize {
        self.id += 1;
        self.id
    }
}

pub fn program(program: &mut Program, session: &mut Session) {
    let mut new_items = Vec::new();

    for top_level in program.items.iter_mut() {
        match top_level {
            TopLevel::FunctionDef(this) => {
                block(&mut this.body, session);
                new_items.push(top_level.clone());
            }
            TopLevel::FunctionProto(_) => {
                new_items.push(top_level.clone());
            }
            TopLevel::Stmt(stmt) => {
                let simplified_stmts = simplify_stmt(stmt, session);
                for stmt in simplified_stmts {
                    new_items.push(TopLevel::Stmt(*stmt));
                }
            }
        }
    }

    program.items = new_items;
}

fn simplify_stmt(stmt_: &mut Stmt, session: &mut Session) -> Vec<Box<Stmt>> {
    match stmt_ {
        Stmt::ExprStmt(expr) => {
            let expr_value = std::mem::replace(expr, Expr::NumInt(0));
            let new_expr = _expr(expr_value);
            *expr = new_expr;
            vec![Box::new(stmt_.clone())]
        }
        Stmt::DeclStmt(decl_stmt) => simplify_decl_stmt(decl_stmt, session)
            .into_iter()
            .map(|x| Stmt::decl_stmt(x))
            .collect(),
        Stmt::Control(control) => {
            control_simplify(control, session);
            vec![Box::new(stmt_.clone())]
        }
        Stmt::Return(ret) => {
            if let Some(value) = &mut ret.value {
                let expr_value = std::mem::replace(value.as_mut(), Expr::NumInt(0));
                let new_expr = _expr(expr_value);
                **value = new_expr;
            }
            vec![Box::new(stmt_.clone())]
        }
        Stmt::Goto(_) => {
            vec![Box::new(stmt_.clone())]
        }
        Stmt::Label(label) => {
            let simplified_inner = simplify_stmt(&mut label.stmt, session);
            if simplified_inner.len() == 1 {
                label.stmt = simplified_inner.into_iter().next().unwrap();
                vec![Box::new(stmt_.clone())]
            } else {
                // If the inner statement was split, we need to handle this case
                // For simplicity, we'll just return the original for now
                vec![Box::new(stmt_.clone())]
            }
        }
        Stmt::Block(this) => {
            block(this, session);
            vec![Box::new(stmt_.clone())]
        }
        Stmt::Break | Stmt::Continue => {
            vec![Box::new(stmt_.clone())]
        }
    }
}

fn simplify_decl_stmt(decl_stmt: &mut DeclStmt, session: &mut Session) -> Vec<DeclStmt> {
    match decl_stmt {
        DeclStmt::InitVec(inits) => {
            for init in inits {
                if let Some(init_data) = &mut init.l {
                    init_data_simplify(init_data, session);
                }
            }
            vec![decl_stmt.clone()]
        }
        DeclStmt::Struct(struct_decl) => {
            vec![decl_stmt.clone()]
        }
        DeclStmt::Union(_) => {
            vec![decl_stmt.clone()]
        }
        DeclStmt::Enum(_) => {
            vec![decl_stmt.clone()]
        }
        DeclStmt::Typedef(typedef) => {
            // Check if this is a typedef of a struct/union/enum with inline definition
            match &mut *typedef.actual_type {
                Type::Struct(struct_def) => {
                    // This is typedef struct name { ... } alias;
                    // Split into: struct name { ... }; and typedef struct name alias;
                    force_name_struct(struct_def, session);
                    let struct_stmt = DeclStmt::Struct(struct_def.clone());

                    let typedef_stmt = DeclStmt::Typedef(Typedef::new(
                        typedef.type_name.clone(),
                        Type::Struct(Struct::new(struct_def.ident.clone(), vec![])),
                    ));

                    vec![struct_stmt, typedef_stmt]
                }
                Type::Union(union_def) => {
                    if union_def.ident.is_some() {
                        // This is typedef union name { ... } alias;
                        // Split into: union name { ... }; and typedef union name alias;

                        let union_stmt = DeclStmt::Union(union_def.clone());

                        let typedef_stmt = DeclStmt::Typedef(Typedef::new(
                            typedef.type_name.clone(),
                            Type::Union(Union::new(union_def.ident.clone(), vec![])),
                        ));

                        vec![union_stmt, typedef_stmt]
                    } else {
                        // Anonymous union, keep as is
                        vec![decl_stmt.clone()]
                    }
                }
                Type::Enum(enum_def) => {
                    if enum_def.ident.is_some() {
                        // This is typedef enum name { ... } alias;
                        // Split into: enum name { ... }; and typedef enum name alias;

                        let enum_stmt = DeclStmt::Enum(enum_def.clone());

                        let typedef_stmt = DeclStmt::Typedef(Typedef::new(
                            typedef.type_name.clone(),
                            Type::Enum(Enum::new(enum_def.ident.clone(), vec![])),
                        ));

                        vec![enum_stmt, typedef_stmt]
                    } else {
                        // Anonymous enum, keep as is
                        vec![decl_stmt.clone()]
                    }
                }
                _ => {
                    // Regular typedef, keep as is
                    vec![decl_stmt.clone()]
                }
            }
        }
    }
}

fn force_name_struct(r#struct: &mut Struct, session: &mut Session) {
    if r#struct.ident.is_none() {
        r#struct.ident = Some(Ident::new(session.id().to_string()))
    }
}

fn block(block: &mut Block, session: &mut Session) {
    let mut new_statements = Vec::new();

    for stmt in block.statements.iter_mut() {
        let simplified_stmts = simplify_stmt(stmt, session);
        new_statements.extend(simplified_stmts);
    }

    block.statements = new_statements;
}

fn stmt(stmt_: &mut Stmt, session: &mut Session) {
    match stmt_ {
        Stmt::ExprStmt(expr) => {
            let expr_value = std::mem::replace(expr, Expr::NumInt(0));
            let new_expr = _expr(expr_value);
            *expr = new_expr;
        }
        Stmt::DeclStmt(decl_stmt) => {
            decl_stmt_simplify(decl_stmt, session);
        }
        Stmt::Control(control) => {
            control_simplify(control, session);
        }
        Stmt::Return(ret) => {
            if let Some(value) = &mut ret.value {
                let expr_value = std::mem::replace(value.as_mut(), Expr::NumInt(0));
                let new_expr = _expr(expr_value);
                **value = new_expr;
            }
        }
        Stmt::Goto(_) => {}
        Stmt::Label(label) => {
            stmt(&mut label.stmt, session);
        }
        Stmt::Block(this) => {
            block(this, session);
        }
        Stmt::Break | Stmt::Continue => {}
    }
}

fn decl_stmt_simplify(decl_stmt: &mut DeclStmt, session: &mut Session) {
    match decl_stmt {
        DeclStmt::InitVec(inits) => {
            for init in inits {
                if let Some(init_data) = &mut init.l {
                    init_data_simplify(init_data, session);
                }
            }
        }
        DeclStmt::Struct(_) => {
            // 構造体のメンバーに初期化式があれば処理（通常のCでは稀）
        }
        DeclStmt::Union(_) => {}
        DeclStmt::Enum(_) => {
            // enumの値部分に式があれば処理（通常は定数だが）
        }
        DeclStmt::Typedef(_) => {}
    }
}

fn init_data_simplify(init_data: &mut InitData, session: &mut Session) {
    match init_data {
        InitData::Expr(expr) => {
            let expr_value = std::mem::replace(expr, Expr::NumInt(0));
            let new_expr = _expr(expr_value);
            *expr = new_expr;
        }
        InitData::Compound(compound) => {
            for data in compound {
                init_data_simplify(data, session);
            }
        }
    }
}

fn control_simplify(control: &mut Control, session: &mut Session) {
    match control {
        Control::If(if_stmt) => {
            // 条件式を簡略化
            let cond_value = std::mem::replace(if_stmt.cond.as_mut(), Expr::NumInt(0));
            let new_cond = _expr(cond_value);
            *if_stmt.cond = new_cond;

            // then分岐を簡略化
            stmt(&mut if_stmt.then_branch, session);

            // else分岐があれば簡略化
            if let Some(else_branch) = &mut if_stmt.else_branch {
                stmt(else_branch, session);
            }
        }
        Control::While(while_stmt) => {
            // 条件式を簡略化
            let cond_value = std::mem::replace(while_stmt.cond.as_mut(), Expr::NumInt(0));
            let new_cond = _expr(cond_value);
            *while_stmt.cond = new_cond;

            // 本体を簡略化
            stmt(&mut while_stmt.body, session);
        }
        Control::DoWhile(do_while_stmt) => {
            // 本体を簡略化
            stmt(&mut do_while_stmt.body, session);

            // 条件式を簡略化
            let cond_value = std::mem::replace(do_while_stmt.cond.as_mut(), Expr::NumInt(0));
            let new_cond = _expr(cond_value);
            *do_while_stmt.cond = new_cond;
        }
        Control::For(for_stmt) => {
            // 初期化式があれば簡略化
            if let Some(init) = &mut for_stmt.init {
                let init_value = std::mem::replace(init.as_mut(), Expr::NumInt(0));
                let new_init = _expr(init_value);
                **init = new_init;
            }

            // 条件式があれば簡略化
            if let Some(cond) = &mut for_stmt.cond {
                let cond_value = std::mem::replace(cond.as_mut(), Expr::NumInt(0));
                let new_cond = _expr(cond_value);
                **cond = new_cond;
            }

            // ステップ式があれば簡略化
            if let Some(step) = &mut for_stmt.step {
                let step_value = std::mem::replace(step.as_mut(), Expr::NumInt(0));
                let new_step = _expr(step_value);
                **step = new_step;
            }

            // 本体を簡略化
            stmt(&mut for_stmt.body, session);
        }
        Control::Switch(switch_stmt) => {
            // 条件式を簡略化
            let cond_value = std::mem::replace(switch_stmt.cond.as_mut(), Expr::NumInt(0));
            let new_cond = _expr(cond_value);
            *switch_stmt.cond = new_cond;

            // 各caseを簡略化
            for case in &mut switch_stmt.cases {
                match case {
                    SwitchCase::Case(case_stmt) => {
                        // case定数式を簡略化
                        let const_expr_value =
                            std::mem::replace(&mut case_stmt.const_expr, Expr::NumInt(0));
                        let new_const_expr = _expr(const_expr_value);
                        case_stmt.const_expr = new_const_expr;

                        // case内の文を簡略化
                        for stmt_box in &mut case_stmt.stmts {
                            stmt(stmt_box, session);
                        }
                    }
                    SwitchCase::Default(default_stmt) => {
                        // default内の文を簡略化
                        for stmt_box in &mut default_stmt.stmts {
                            stmt(stmt_box, session);
                        }
                    }
                }
            }
        }
    }
}

fn _expr(expr: Expr) -> Expr {
    match expr {
        Expr::Assign(this) => assign(this),
        Expr::Unary(this) => unary(this),
        Expr::Postfix(this) => postfix(this),
        Expr::Binary(this) => binary(this),
        Expr::Call(this) => call(this),
        Expr::Subscript(this) => subscript(this),
        Expr::MemberAccess(this) => member_access(this),
        Expr::Ternary(this) => ternary(this),
        Expr::Sizeof(this) => sizeof(this),
        Expr::Cast(this) => cast(this),
        Expr::Comma(this) => comma(this),
        // 以下は変換不要なのでそのまま返す
        Expr::Char(this) => Expr::Char(this),
        Expr::String(this) => Expr::String(this),
        Expr::Ident(this) => Expr::Ident(this),
        Expr::NumInt(this) => Expr::NumInt(this),
        Expr::NumFloat(this) => Expr::NumFloat(this),
    }
}

fn postfix(postfix: Postfix) -> Expr {
    let simplified_expr = Box::new(_expr(*postfix.expr));

    Expr::comma(vec![
        *Expr::assign(
            AssignOp::Equal,
            simplified_expr.clone(),
            Expr::binary(
                if postfix.op == PostfixOp::plus_plus() {
                    BinaryOp::plus()
                } else {
                    BinaryOp::minus()
                },
                simplified_expr.clone(),
                Box::new(Expr::NumInt(1)),
            ),
        ),
        *Expr::binary(
            if postfix.op == PostfixOp::plus_plus() {
                BinaryOp::minus()
            } else {
                BinaryOp::plus()
            },
            simplified_expr,
            Box::new(Expr::NumInt(1)),
        ),
    ])
}

fn unary(unary: Unary) -> Expr {
    match unary.op {
        UnaryOp::Minus => {
            let target = *unary.expr;
            Expr::Binary(Binary {
                lhs: Box::new(Expr::NumInt(0)),
                op: BinaryOp::minus(),
                rhs: Box::new(_expr(target)),
            })
        }
        UnaryOp::MinusMinus | UnaryOp::PlusPlus => {
            let simplified_expr = Box::new(_expr(*unary.expr));

            *Expr::assign(
                AssignOp::equal(),
                simplified_expr.clone(),
                Expr::binary(
                    if unary.op == UnaryOp::PlusPlus {
                        BinaryOp::plus()
                    } else {
                        BinaryOp::minus()
                    },
                    simplified_expr,
                    Box::new(Expr::NumInt(1)),
                ),
            )
        }
        _ => {
            let simplified_expr = Box::new(_expr(*unary.expr));
            *Expr::unary(unary.op, simplified_expr)
        }
    }
}

fn assign(assign: Assign) -> Expr {
    let simplified_lhs = Box::new(_expr(*assign.lhs));
    let simplified_rhs = Box::new(_expr(*assign.rhs));

    if assign.op == AssignOp::Equal {
        return *Expr::assign(assign.op, simplified_lhs, simplified_rhs);
    }

    // 複合代入演算子を基本的な代入に変換
    let assign_expr = Expr::assign(
        AssignOp::equal(),
        simplified_lhs.clone(),
        Expr::binary(
            match assign.op {
                AssignOp::PlusEqual => BinaryOp::plus(),
                AssignOp::MinusEqual => BinaryOp::minus(),
                AssignOp::AsteriskEqual => BinaryOp::asterisk(),
                AssignOp::SlashEqual => BinaryOp::slash(),
                AssignOp::PercentEqual => BinaryOp::percent(),
                AssignOp::CaretEqual => BinaryOp::caret(),
                AssignOp::PipeEqual => BinaryOp::pipe(),
                AssignOp::LessLessEqual => BinaryOp::less_less(),
                AssignOp::GreaterGreaterEqual => BinaryOp::greater_greater(),
                AssignOp::AmpersandEqual => BinaryOp::ampersand(),
                AssignOp::Equal => unreachable!(),
            },
            simplified_lhs,
            simplified_rhs,
        ),
    );

    *assign_expr
}

fn binary(binary: Binary) -> Expr {
    *Expr::binary(
        binary.op,
        Box::new(_expr(*binary.lhs)),
        Box::new(_expr(*binary.rhs)),
    )
}

fn call(call: Call) -> Expr {
    Expr::call(
        _expr(*call.func),
        call.args
            .into_iter()
            .map(|arg| Box::new(_expr(*arg)))
            .collect(),
    )
}

fn subscript(subscript: Subscript) -> Expr {
    Expr::subscript(_expr(*subscript.name), _expr(*subscript.index))
}

fn member_access(member_access: MemberAccess) -> Expr {
    match member_access.kind {
        MemberAccessOp::MinusGreater => Expr::member_access(
            *Expr::unary(UnaryOp::asterisk(), member_access.base),
            member_access.member,
            MemberAccessOp::dot(),
        ),
        _ => Expr::member_access(
            _expr(*member_access.base),
            member_access.member,
            member_access.kind,
        ),
    }
}

fn ternary(ternary: Ternary) -> Expr {
    *Expr::ternary(
        Box::new(_expr(*ternary.cond)),
        _expr(*ternary.then_branch),
        _expr(*ternary.else_branch),
    )
}

fn sizeof(sizeof: Sizeof) -> Expr {
    let new_sizeof = match sizeof {
        Sizeof::Type(ty) => Sizeof::Type(ty),
        Sizeof::Expr(expr) => Sizeof::Expr(Box::new(_expr(*expr))),
    };
    *Expr::sizeof(new_sizeof)
}

fn cast(cast: Cast) -> Expr {
    *Expr::cast(*cast.r#type, _expr(*cast.expr))
}

fn comma(comma: Comma) -> Expr {
    Expr::comma(comma.assigns.into_iter().map(_expr).collect())
}
