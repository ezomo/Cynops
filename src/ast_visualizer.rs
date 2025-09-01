use crate::ast::*;
use crate::sema::ast::TypedExpr;

pub fn visualize_program(program: &Program) {
    println!("Program");
    for (i, item) in program.items.iter().enumerate() {
        let is_last = i == program.items.len() - 1;
        match item {
            TopLevel::FunctionDef(func) => {
                visualize_function_def(func, 1, is_last, vec![]);
            }
            TopLevel::FunctionProto(proto) => {
                visualize_function_proto(proto, 1, is_last, vec![]);
            }
            TopLevel::Stmt(stmt) => {
                visualize_stmt(stmt, 1, is_last, vec![]);
            }
        }
    }
}

fn visualize_function_proto(
    proto: &FunctionProto,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    print_branch(
        "FunctionProto",
        &proto.sig.ident.name,
        indent,
        is_last,
        &prefix,
    );

    print_branch(
        "Type",
        &proto.sig.ty.to_rust_format(),
        indent + 1,
        true,
        &extend_prefix(&prefix, !is_last),
    );
}

fn visualize_function_def(func: &FunctionDef, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch(
        "FunctionDef",
        &func.sig.ident.name,
        indent,
        is_last,
        &prefix,
    );

    let has_body = !func.body.statements.is_empty();
    let has_params = !func.param_names.is_empty();
    let total_items = 2 + if has_body { 1 } else { 0 } + if has_params { 1 } else { 0 }; // ReturnType, Params, Body

    let mut current_item = 0;
    current_item += 1;
    print_branch(
        "Type",
        &func.sig.ty.to_rust_format(),
        indent + 1,
        current_item == total_items,
        &extend_prefix(&prefix, !is_last),
    );

    if has_params {
        current_item += 1;
        print_branch(
            "Params",
            "",
            indent + 1,
            current_item == total_items,
            &extend_prefix(&prefix, !is_last),
        );
        for (i, param) in func.param_names.iter().enumerate() {
            let is_last_param = i == func.param_names.len() - 1;
            print_branch(
                "Param",
                &param.name,
                indent + 2,
                is_last_param,
                &extend_prefix(
                    &extend_prefix(&prefix, !is_last),
                    current_item < total_items,
                ),
            );
        }
    }

    if has_body {
        current_item += 1;
        print_branch(
            "Body",
            "",
            indent + 1,
            current_item == total_items,
            &extend_prefix(&prefix, !is_last),
        );
        for (i, stmt) in func.body.statements.iter().enumerate() {
            let last_stmt = i == func.body.statements.len() - 1;
            visualize_stmt(
                stmt,
                indent + 2,
                last_stmt,
                extend_prefix(&extend_prefix(&prefix, !is_last), false),
            );
        }
    }
}

fn visualize_stmt(stmt: &Stmt, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match stmt {
        Stmt::Return(ret) => {
            if let Some(expr) = &ret.value {
                print_branch("Return", "", indent, is_last, &prefix);
                visualize_expr(expr, indent + 1, true, extend_prefix(&prefix, !is_last));
            } else {
                print_branch("Return", "(void)", indent, is_last, &prefix);
            }
        }
        Stmt::DeclStmt(decl_stmt) => match decl_stmt {
            DeclStmt::InitVec(inits) => {
                print_branch("DeclStmt", "InitVec", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                for (i, init) in inits.iter().enumerate() {
                    let is_last_init = i == inits.len() - 1;
                    visualize_init(init, indent + 1, is_last_init, next_prefix.clone());
                }
            }
            DeclStmt::Struct(struct_decl) => {
                print_branch(
                    "StructDeclStmt",
                    &format!(
                        "{}",
                        struct_decl
                            .ident
                            .as_ref()
                            .map(|n| n.name.as_str())
                            .unwrap_or("anonymous")
                    ),
                    indent,
                    is_last,
                    &prefix,
                );

                let next_prefix = extend_prefix(&prefix, !is_last);
                print_branch("Members", "", indent + 1, true, &next_prefix);

                for (i, member) in struct_decl.member.iter().enumerate() {
                    let is_last_member = i == struct_decl.member.len() - 1;
                    visualize_member_decl(
                        member,
                        indent + 2,
                        is_last_member,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
            DeclStmt::Union(union_decl) => {
                print_branch(
                    "UnionDeclStmt",
                    &format!(
                        "{}",
                        union_decl
                            .ident
                            .as_ref()
                            .map(|n| n.name.as_str())
                            .unwrap_or("anonymous")
                    ),
                    indent,
                    is_last,
                    &prefix,
                );

                let next_prefix = extend_prefix(&prefix, !is_last);
                print_branch("Members", "", indent + 1, true, &next_prefix);

                for (i, member) in union_decl.member.iter().enumerate() {
                    let is_last_member = i == union_decl.member.len() - 1;
                    visualize_member_decl(
                        member,
                        indent + 2,
                        is_last_member,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
            DeclStmt::Enum(enum_decl) => {
                print_branch(
                    "EnumDeclStmt",
                    &format!(
                        "{}",
                        enum_decl
                            .ident
                            .as_ref()
                            .map(|n| n.name.as_str())
                            .unwrap_or("anonymous")
                    ),
                    indent,
                    is_last,
                    &prefix,
                );

                let next_prefix = extend_prefix(&prefix, !is_last);
                print_branch("Variants", "", indent + 1, true, &next_prefix);

                for (i, variant) in enum_decl.variants.iter().enumerate() {
                    let is_last_variant = i == enum_decl.variants.len() - 1;
                    let variant_info = match &variant.value {
                        Some(value) => format!("{} = {}", variant.ident.name, value),
                        None => variant.ident.name.clone(),
                    };
                    print_branch(
                        "Variant",
                        &variant_info,
                        indent + 2,
                        is_last_variant,
                        &extend_prefix(&next_prefix, false),
                    );
                }
            }
            DeclStmt::Typedef(typedef) => {
                print_branch(
                    "TypedefDeclStmt",
                    &typedef.type_name.name,
                    indent,
                    is_last,
                    &prefix,
                );

                let next_prefix = extend_prefix(&prefix, !is_last);
                print_branch("ActualType", "", indent + 1, true, &next_prefix);
                visualize_type(
                    &typedef.actual_type,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, false),
                );
            }
        },
        Stmt::ExprStmt(expr) => {
            print_branch("ExprStmt", "", indent, is_last, &prefix);
            visualize_expr(expr, indent + 1, true, extend_prefix(&prefix, !is_last));
        }
        Stmt::Control(control) => match control {
            Control::If(if_stmt) => {
                print_branch("If", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                print_branch("Condition", "", indent + 1, false, &next_prefix);
                visualize_expr(
                    &if_stmt.cond,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, true),
                );

                print_branch(
                    "Then",
                    "",
                    indent + 1,
                    if_stmt.else_branch.is_none(),
                    &next_prefix,
                );
                visualize_stmt(
                    &if_stmt.then_branch,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, if_stmt.else_branch.is_some()),
                );

                if let Some(else_branch) = &if_stmt.else_branch {
                    print_branch("Else", "", indent + 1, true, &next_prefix);
                    visualize_stmt(
                        else_branch,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
            Control::While(while_stmt) => {
                print_branch("While", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                print_branch("Condition", "", indent + 1, false, &next_prefix);
                visualize_expr(
                    &while_stmt.cond,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, true),
                );

                print_branch("Body", "", indent + 1, true, &next_prefix);
                visualize_stmt(
                    &while_stmt.body,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, false),
                );
            }
            Control::For(for_stmt) => {
                print_branch("For", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                let has_init = for_stmt.init.is_some();
                let has_cond = for_stmt.cond.is_some();
                let has_step = for_stmt.step.is_some();
                let mut remaining_items = 1; // body is always present
                if has_step {
                    remaining_items += 1;
                }
                if has_cond {
                    remaining_items += 1;
                }
                if has_init {
                    remaining_items += 1;
                }

                if let Some(init) = &for_stmt.init {
                    remaining_items -= 1;
                    print_branch("Init", "", indent + 1, remaining_items == 0, &next_prefix);
                    visualize_expr(
                        init,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, remaining_items > 0),
                    );
                }
                if let Some(cond) = &for_stmt.cond {
                    remaining_items -= 1;
                    print_branch(
                        "Condition",
                        "",
                        indent + 1,
                        remaining_items == 0,
                        &next_prefix,
                    );
                    visualize_expr(
                        cond,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, remaining_items > 0),
                    );
                }
                if let Some(step) = &for_stmt.step {
                    remaining_items -= 1;
                    print_branch("Step", "", indent + 1, remaining_items == 0, &next_prefix);
                    visualize_expr(
                        step,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, remaining_items > 0),
                    );
                }
                print_branch("Body", "", indent + 1, true, &next_prefix);
                visualize_stmt(
                    &for_stmt.body,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, false),
                );
            }
            Control::DoWhile(do_while_stmt) => {
                print_branch("DoWhile", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                print_branch("Body", "", indent + 1, false, &next_prefix);
                visualize_stmt(
                    &do_while_stmt.body,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, true),
                );

                print_branch("Condition", "", indent + 1, true, &next_prefix);
                visualize_expr(
                    &do_while_stmt.cond,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, false),
                );
            }
            Control::Switch(switch_stmt) => {
                print_branch("Switch", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                print_branch(
                    "Condition",
                    "",
                    indent + 1,
                    switch_stmt.cases.is_empty(),
                    &next_prefix,
                );
                visualize_expr(
                    &switch_stmt.cond,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, !switch_stmt.cases.is_empty()),
                );

                if !switch_stmt.cases.is_empty() {
                    print_branch("Cases", "", indent + 1, true, &next_prefix);
                    for (i, case) in switch_stmt.cases.iter().enumerate() {
                        let last_case = i == switch_stmt.cases.len() - 1;
                        visualize_switch_case(
                            case,
                            indent + 2,
                            last_case,
                            extend_prefix(&next_prefix, false),
                        );
                    }
                }
            }
        },
        Stmt::Block(block) => {
            if block.statements.is_empty() {
                print_branch("Block", "(empty)", indent, is_last, &prefix);
            } else {
                print_branch("Block", "", indent, is_last, &prefix);
                for (i, stmt) in block.statements.iter().enumerate() {
                    let last_stmt = i == block.statements.len() - 1;
                    visualize_stmt(
                        stmt,
                        indent + 1,
                        last_stmt,
                        extend_prefix(&prefix, !is_last),
                    );
                }
            }
        }
        Stmt::Break => {
            print_branch("Break", "", indent, is_last, &prefix);
        }
        Stmt::Continue => {
            print_branch("Continue", "", indent, is_last, &prefix);
        }
        Stmt::Goto(goto) => {
            print_branch(
                "Goto",
                &format!("→ {}", goto.label.name),
                indent,
                is_last,
                &prefix,
            );
        }
        Stmt::Label(label) => {
            print_branch(
                "Label",
                &format!("{}:", label.name.name),
                indent,
                is_last,
                &prefix,
            );
            visualize_stmt(
                &label.stmt,
                indent + 1,
                true,
                extend_prefix(&prefix, !is_last),
            );
        }
    }
}

fn visualize_init(init: &Init, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch("Init", &init.r.ident.name, indent, is_last, &prefix);
    let next_prefix = extend_prefix(&prefix, !is_last);

    print_branch(
        "Type",
        &init.r.ty.to_rust_format(),
        indent + 1,
        init.l.is_none(),
        &next_prefix,
    );

    if let Some(init_data) = &init.l {
        print_branch("Initializer", "", indent + 1, true, &next_prefix);
        visualize_init_data(
            init_data,
            indent + 2,
            true,
            extend_prefix(&next_prefix, false),
        );
    }
}

fn visualize_init_data(init_data: &InitData, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match init_data {
        InitData::Expr(expr) => {
            visualize_expr(expr, indent, is_last, prefix);
        }
        InitData::Compound(list) => {
            if list.is_empty() {
                print_branch("Compound", "(empty)", indent, is_last, &prefix);
            } else {
                print_branch("Compound", "", indent, is_last, &prefix);
                for (i, init) in list.iter().enumerate() {
                    let last_init = i == list.len() - 1;
                    visualize_init_data(
                        init,
                        indent + 1,
                        last_init,
                        extend_prefix(&prefix, !is_last),
                    );
                }
            }
        }
    }
}

fn visualize_member_decl(member: &MemberDecl, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch(
        "Member",
        &format!("{}: {}", member.ident.name, member.ty.to_rust_format()),
        indent,
        is_last,
        &prefix,
    );
}

fn visualize_type(ty: &Type, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch(&ty.to_rust_format(), "", indent, is_last, &prefix)
}

fn visualize_switch_case(case: &SwitchCase, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match case {
        SwitchCase::Case(case_stmt) => {
            print_branch("Case", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch(
                "Value",
                "",
                indent + 1,
                case_stmt.stmts.is_empty(),
                &next_prefix,
            );

            // TODO
            // print_branch(
            //     "Value",
            //     "",
            //     indent + 1,
            //     case_stmt.stmts.is_empty(),
            //     &next_prefix,
            // );

            if !case_stmt.stmts.is_empty() {
                print_branch("Statements", "", indent + 1, true, &next_prefix);
                for (i, stmt) in case_stmt.stmts.iter().enumerate() {
                    let last_stmt = i == case_stmt.stmts.len() - 1;
                    visualize_stmt(
                        stmt,
                        indent + 2,
                        last_stmt,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
        }
        SwitchCase::Default(default_case) => {
            if default_case.stmts.is_empty() {
                print_branch("Default", "(empty)", indent, is_last, &prefix);
            } else {
                print_branch("Default", "", indent, is_last, &prefix);
                let next_prefix = extend_prefix(&prefix, !is_last);

                print_branch("Statements", "", indent + 1, true, &next_prefix);
                for (i, stmt) in default_case.stmts.iter().enumerate() {
                    let last_stmt = i == default_case.stmts.len() - 1;
                    visualize_stmt(
                        stmt,
                        indent + 2,
                        last_stmt,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
        }
    }
}

fn visualize_typed_expr(typed_expr: &TypedExpr, indent: usize, is_last: bool, prefix: Vec<bool>) {
    // Show type information first
    print_branch(
        "TypedExpr",
        &format!("Type: {}", typed_expr.r#type.to_rust_format()),
        indent,
        is_last,
        &prefix,
    );

    // Then show the expression
    visualize_sema_expr(
        &typed_expr.r#expr,
        indent + 1,
        true,
        extend_prefix(&prefix, !is_last),
    );
}

fn visualize_sema_expr(
    expr: &crate::sema::ast::SemaExpr,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    use crate::sema::ast::SemaExpr;

    match expr {
        SemaExpr::Comma(c) => {
            print_branch("Comma", "", indent, is_last, &prefix);
            let new_prefix = extend_prefix(&prefix, !is_last);

            for (i, expr) in c.assigns.iter().enumerate() {
                let is_last_expr = i == c.assigns.len() - 1;
                visualize_typed_expr(expr, indent + 1, is_last_expr, new_prefix.clone());
            }
        }
        SemaExpr::NumInt(n) => {
            print_branch("Number", &n.to_string(), indent, is_last, &prefix);
        }
        SemaExpr::NumFloat(n) => {
            print_branch("Number", &n.to_string(), indent, is_last, &prefix);
        }
        SemaExpr::Char(c) => {
            print_branch("Character", &format!("'{:?}'", c), indent, is_last, &prefix);
        }
        SemaExpr::Ident(name) => {
            print_branch("Identifier", &name.name, indent, is_last, &prefix);
        }
        SemaExpr::Binary(binary) => {
            print_branch(
                "Binary",
                &format!("{:?}", binary.op),
                indent,
                is_last,
                &prefix,
            );
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Left", "", indent + 1, false, &new_prefix);
            visualize_typed_expr(
                &binary.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Right", "", indent + 1, true, &new_prefix);
            visualize_typed_expr(
                &binary.rhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        SemaExpr::Unary(unary) => {
            print_branch(
                "Unary",
                &format!("{:?}", unary.op),
                indent,
                is_last,
                &prefix,
            );
            print_branch(
                "Operand",
                "",
                indent + 1,
                true,
                &extend_prefix(&prefix, !is_last),
            );
            visualize_typed_expr(
                &unary.expr,
                indent + 2,
                true,
                extend_prefix(&extend_prefix(&prefix, !is_last), false),
            );
        }
        SemaExpr::Call(call) => {
            print_branch("FunctionCall", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Function", "", indent + 1, false, &next_prefix);
            visualize_typed_expr(
                &call.func,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            if !call.args.is_empty() {
                print_branch("Arguments", "", indent + 1, true, &next_prefix);
                for (i, arg) in call.args.iter().enumerate() {
                    let last = i == call.args.len() - 1;
                    visualize_typed_expr(
                        arg,
                        indent + 2,
                        last,
                        extend_prefix(&extend_prefix(&next_prefix, false), false),
                    );
                }
            } else {
                print_branch("Arguments", "(empty)", indent + 1, true, &next_prefix);
            }
        }
        SemaExpr::Assign(assign) => {
            print_branch(
                "Assignment",
                &format!("{:?}", assign.op),
                indent,
                is_last,
                &prefix,
            );
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Left", "", indent + 1, false, &new_prefix);
            visualize_typed_expr(
                &assign.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Right", "", indent + 1, true, &new_prefix);
            visualize_typed_expr(
                &assign.rhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        SemaExpr::Ternary(ternary) => {
            print_branch("Ternary", "", indent, is_last, &prefix);
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Condition", "", indent + 1, false, &new_prefix);
            visualize_typed_expr(
                &ternary.cond,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Then", "", indent + 1, false, &new_prefix);
            visualize_typed_expr(
                &ternary.then_branch,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Else", "", indent + 1, true, &new_prefix);
            visualize_typed_expr(
                &ternary.else_branch,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        SemaExpr::Subscript(array_access) => {
            print_branch("ArrayAccess", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Array", "", indent + 1, false, &next_prefix);
            visualize_typed_expr(
                &array_access.subject,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            print_branch("Index", "", indent + 1, true, &next_prefix);
            visualize_typed_expr(
                &array_access.index,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
        SemaExpr::MemberAccess(member_access) => {
            print_branch(
                "MemberAccess",
                &format!("{:?}", member_access.kind),
                indent,
                is_last,
                &prefix,
            );
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Base", "", indent + 1, false, &next_prefix);
            visualize_typed_expr(
                &member_access.base,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            print_branch("Member", "", indent + 1, true, &next_prefix);
            let member_expr = TypedExpr::new(
                crate::ast::Type::Void, // Placeholder type for member identifier
                SemaExpr::Ident(member_access.member.clone()),
            );
            visualize_typed_expr(
                &member_expr,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
        SemaExpr::Sizeof(sizeof) => {
            print_branch("Sizeof", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            match sizeof {
                crate::sema::ast::Sizeof::Type(ty) => {
                    print_branch("Type", &format!("{:?}", ty), indent + 1, true, &next_prefix);
                }
                crate::sema::ast::Sizeof::TypedExpr(expr) => {
                    print_branch("Expression", "", indent + 1, true, &next_prefix);
                    visualize_typed_expr(
                        expr,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, false),
                    );
                }
            }
        }
        SemaExpr::Cast(cast) => {
            print_branch("Cast", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch(
                "Type",
                &format!("{:?}", cast.r#type),
                indent + 1,
                false,
                &next_prefix,
            );
            print_branch("Expression", "", indent + 1, true, &next_prefix);
            visualize_typed_expr(
                &cast.expr,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
    }
}

fn print_branch(label: &str, value: &str, _indent: usize, is_last: bool, prefix: &[bool]) {
    // Draw the tree structure
    for &p in prefix {
        if p {
            print!("│   ");
        } else {
            print!("    ");
        }
    }

    // Draw the current branch
    if is_last {
        print!("└── ");
    } else {
        print!("├── ");
    }

    // Print the label and value
    if value.is_empty() {
        println!("{}", label);
    } else {
        println!("{}: {}", label, value);
    }
}

fn extend_prefix(prefix: &[bool], has_more: bool) -> Vec<bool> {
    let mut new = prefix.to_vec();
    new.push(has_more);
    new
}

fn visualize_expr(expr: &Expr, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match expr {
        Expr::Comma(c) => {
            print_branch("Comma", "", indent, is_last, &prefix);
            let new_prefix = extend_prefix(&prefix, !is_last);

            for (i, expr) in c.assigns.iter().enumerate() {
                let is_last_expr = i == c.assigns.len() - 1;
                visualize_expr(expr, indent + 1, is_last_expr, new_prefix.clone());
            }
        }
        Expr::NumInt(n) => {
            print_branch("Number", &n.to_string(), indent, is_last, &prefix);
        }
        Expr::NumFloat(n) => {
            print_branch("Number", &n.to_string(), indent, is_last, &prefix);
        }
        Expr::Char(c) => {
            print_branch("Character", &format!("'{}'", c), indent, is_last, &prefix);
        }
        Expr::String(c) => {
            print_branch("String", &format!("\"{:?}\"", c), indent, is_last, &prefix);
        }
        Expr::Ident(name) => {
            print_branch("Identifier", &name.name, indent, is_last, &prefix);
        }
        Expr::Binary(binary) => {
            print_branch(
                "Binary",
                &format!("{:?}", binary.op),
                indent,
                is_last,
                &prefix,
            );
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Left", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &binary.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Right", "", indent + 1, true, &new_prefix);
            visualize_expr(
                &binary.rhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        Expr::Unary(unary) => {
            print_branch(
                "Unary",
                &format!("{:?}", unary.op),
                indent,
                is_last,
                &prefix,
            );
            print_branch(
                "Operand",
                "",
                indent + 1,
                true,
                &extend_prefix(&prefix, !is_last),
            );
            visualize_expr(
                &unary.expr,
                indent + 2,
                true,
                extend_prefix(&extend_prefix(&prefix, !is_last), false),
            );
        }
        Expr::Postfix(postfix) => {
            print_branch(
                "Postfix",
                &format!("{:?}", postfix.op),
                indent,
                is_last,
                &prefix,
            );
            print_branch(
                "Operand",
                "",
                indent + 1,
                true,
                &extend_prefix(&prefix, !is_last),
            );
            visualize_expr(
                &postfix.expr,
                indent + 2,
                true,
                extend_prefix(&extend_prefix(&prefix, !is_last), false),
            );
        }
        Expr::Call(call) => {
            print_branch("FunctionCall", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Function", "", indent + 1, false, &next_prefix);
            visualize_expr(
                &call.func,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            if !call.args.is_empty() {
                print_branch("Arguments", "", indent + 1, true, &next_prefix);
                for (i, arg) in call.args.iter().enumerate() {
                    let last = i == call.args.len() - 1;
                    visualize_expr(
                        arg,
                        indent + 2,
                        last,
                        extend_prefix(&extend_prefix(&next_prefix, false), false),
                    );
                }
            } else {
                print_branch("Arguments", "(empty)", indent + 1, true, &next_prefix);
            }
        }
        Expr::Assign(assign) => {
            print_branch(
                "Assignment",
                &format!("{:?}", assign.op),
                indent,
                is_last,
                &prefix,
            );
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Left", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &assign.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Right", "", indent + 1, true, &new_prefix);
            visualize_expr(
                &assign.rhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        Expr::Ternary(ternary) => {
            print_branch("Ternary", "", indent, is_last, &prefix);
            let new_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Condition", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &ternary.cond,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Then", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &ternary.then_branch,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );

            print_branch("Else", "", indent + 1, true, &new_prefix);
            visualize_expr(
                &ternary.else_branch,
                indent + 2,
                true,
                extend_prefix(&new_prefix, false),
            );
        }
        Expr::Subscript(array_access) => {
            print_branch("ArrayAccess", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Array", "", indent + 1, false, &next_prefix);
            visualize_expr(
                &array_access.name,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            print_branch("Index", "", indent + 1, true, &next_prefix);
            visualize_expr(
                &array_access.index,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
        Expr::MemberAccess(member_access) => {
            print_branch(
                "MemberAccess",
                &format!("{:?}", member_access.kind),
                indent,
                is_last,
                &prefix,
            );
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Base", "", indent + 1, false, &next_prefix);
            visualize_expr(
                &member_access.base,
                indent + 2,
                true,
                extend_prefix(&next_prefix, true),
            );

            print_branch("Member", "", indent + 1, true, &next_prefix);
            let member_expr = Expr::Ident(member_access.member.clone());
            visualize_expr(
                &member_expr,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
        Expr::Sizeof(sizeof) => {
            print_branch("Sizeof", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            match sizeof {
                Sizeof::Type(ty) => {
                    print_branch("Type", &format!("{:?}", ty), indent + 1, true, &next_prefix);
                }
                Sizeof::Expr(expr) => {
                    print_branch("Expression", "", indent + 1, true, &next_prefix);
                    visualize_expr(expr, indent + 2, true, extend_prefix(&next_prefix, false));
                }
            }
        }
        Expr::Cast(cast) => {
            print_branch("Cast", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch(
                "Type",
                &format!("{:?}", cast.r#type),
                indent + 1,
                false,
                &next_prefix,
            );
            print_branch("Expression", "", indent + 1, true, &next_prefix);
            visualize_expr(
                &cast.expr,
                indent + 2,
                true,
                extend_prefix(&next_prefix, false),
            );
        }
    }
}
