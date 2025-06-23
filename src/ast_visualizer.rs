use crate::symbols::*;

pub fn visualize_program(program: &Program) {
    println!("Program");
    for (i, item) in program.items.iter().enumerate() {
        let is_last = i == program.items.len() - 1;
        match item {
            TopLevel::FunctionDef(func) => {
                visualize_function_def(func, 1, is_last, vec![]);
            }
            TopLevel::Stmt(stmt) => {
                visualize_stmt(stmt, 1, is_last, vec![]);
            }
            TopLevel::FunctionProto(proto) => {
                visualize_function_proto(proto, 1, is_last, vec![]);
            }
        }
    }
}

// Helper function to extract function name from declarator
fn extract_function_name(declarator: &Declarator) -> String {
    match declarator {
        Declarator::Direct(direct) => extract_direct_declarator_name(direct),
        Declarator::Pointer(pointer) => extract_direct_declarator_name(&pointer.inner),
    }
}

fn extract_direct_declarator_name(direct: &DirectDeclarator) -> String {
    match direct {
        DirectDeclarator::Ident(ident) => ident.name.clone(),
        DirectDeclarator::Paren(decl) => extract_function_name(decl),
        DirectDeclarator::Array { base, .. } => extract_direct_declarator_name(base),
        DirectDeclarator::Func { base, .. } => extract_direct_declarator_name(base),
    }
}

// Helper function to extract parameters from declarator
fn extract_function_params(declarator: &Declarator) -> Option<&ParamList> {
    match declarator {
        Declarator::Direct(direct) => extract_direct_declarator_params(direct),
        Declarator::Pointer(pointer) => extract_direct_declarator_params(&pointer.inner),
    }
}

fn extract_direct_declarator_params(direct: &DirectDeclarator) -> Option<&ParamList> {
    match direct {
        DirectDeclarator::Func { params, .. } => params.as_ref(),
        DirectDeclarator::Paren(decl) => extract_function_params(decl),
        DirectDeclarator::Array { base, .. } => extract_direct_declarator_params(base),
        _ => None,
    }
}

fn visualize_function_proto(
    proto: &FunctionProto,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    let func_name = extract_function_name(&proto.sig.declarator);
    print_branch("FunctionProto", &func_name, indent, is_last, &prefix);

    let params = extract_function_params(&proto.sig.declarator);
    let has_params = match params {
        Some(ParamList::Void) => false,
        Some(ParamList::Params(params)) => !params.is_empty(),
        None => false,
    };
    let total_items = 1 + if has_params { 1 } else { 0 }; // ReturnType, Params

    print_branch(
        "ReturnType",
        &format!("{:?}", proto.sig.ret_type),
        indent + 1,
        total_items == 1,
        &extend_prefix(&prefix, !is_last),
    );

    if has_params {
        print_branch(
            "Params",
            "",
            indent + 1,
            true,
            &extend_prefix(&prefix, !is_last),
        );
        if let Some(ParamList::Params(params)) = params {
            for (i, param) in params.iter().enumerate() {
                let is_last_param = i == params.len() - 1;
                print_branch(
                    "Param",
                    &format!("{:?} {:?}", param.ty, param.name),
                    indent + 2,
                    is_last_param,
                    &extend_prefix(&extend_prefix(&prefix, !is_last), false),
                );
            }
        }
    } else {
        print_branch(
            "Params",
            "void",
            indent + 1,
            true,
            &extend_prefix(&prefix, !is_last),
        );
    }
}

fn visualize_function_def(func: &FunctionDef, indent: usize, is_last: bool, prefix: Vec<bool>) {
    let func_name = extract_function_name(&func.sig.declarator);
    print_branch("FunctionDef", &func_name, indent, is_last, &prefix);

    let has_body = !func.body.statements.is_empty();
    let total_items = 2 + if has_body { 1 } else { 0 }; // ReturnType, Params, optionally Body

    print_branch(
        "ReturnType",
        &format!("{:?}", func.sig.ret_type),
        indent + 1,
        total_items == 1,
        &extend_prefix(&prefix, !is_last),
    );

    let params = extract_function_params(&func.sig.declarator);
    match params {
        Some(ParamList::Void) | None => {
            print_branch(
                "Params",
                "void",
                indent + 1,
                !has_body,
                &extend_prefix(&prefix, !is_last),
            );
        }
        Some(ParamList::Params(params)) => {
            let param_count = params.len();
            if param_count > 0 {
                print_branch(
                    "Params",
                    "",
                    indent + 1,
                    !has_body,
                    &extend_prefix(&prefix, !is_last),
                );
                for (i, param) in params.iter().enumerate() {
                    let is_last_param = i == param_count - 1;
                    print_branch(
                        "Param",
                        &format!("{:?} {}", param.ty, extract_function_name(&param.name)),
                        indent + 2,
                        is_last_param,
                        &extend_prefix(&extend_prefix(&prefix, !is_last), !has_body),
                    );
                }
            } else {
                print_branch(
                    "Params",
                    "(empty)",
                    indent + 1,
                    !has_body,
                    &extend_prefix(&prefix, !is_last),
                );
            }
        }
    }

    if has_body {
        print_branch(
            "Body",
            "",
            indent + 1,
            true,
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
        Stmt::DeclStmt(decl_stmt) => {
            print_branch(
                "DeclStmt",
                &format!("{:?}", decl_stmt.ty),
                indent,
                is_last,
                &prefix,
            );

            let next_prefix = extend_prefix(&prefix, !is_last);
            print_branch("Declarators", "", indent + 1, true, &next_prefix);

            for (i, declarator) in decl_stmt.declarators.iter().enumerate() {
                let is_last_declarator = i == decl_stmt.declarators.len() - 1;
                visualize_init_declarator(
                    declarator,
                    indent + 2,
                    is_last_declarator,
                    extend_prefix(&next_prefix, false),
                );
            }
        }
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

fn visualize_init_declarator(
    init_decl: &InitDeclarator,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    let next_prefix = extend_prefix(&prefix, !is_last);

    if let Some(init) = &init_decl.init {
        print_branch("InitDeclarator", "", indent, is_last, &prefix);

        print_branch("Declarator", "", indent + 1, false, &next_prefix);
        visualize_declarator(
            &init_decl.declarator,
            indent + 2,
            true,
            extend_prefix(&next_prefix, true),
        );

        print_branch("Initializer", "", indent + 1, true, &next_prefix);
        visualize_initializer(init, indent + 2, true, extend_prefix(&next_prefix, false));
    } else {
        print_branch("Declarator", "", indent, is_last, &prefix);
        visualize_declarator(
            &init_decl.declarator,
            indent + 1,
            true,
            extend_prefix(&prefix, !is_last),
        );
    }
}

fn visualize_declarator(declarator: &Declarator, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match declarator {
        Declarator::Pointer(pointer) => {
            print_branch(
                "Pointer",
                &format!("level: {}", pointer.level),
                indent,
                is_last,
                &prefix,
            );
            visualize_direct_declarator(
                &pointer.inner,
                indent + 1,
                true,
                extend_prefix(&prefix, !is_last),
            );
        }
        Declarator::Direct(direct) => {
            visualize_direct_declarator(direct, indent, is_last, prefix);
        }
    }
}

fn visualize_direct_declarator(
    direct_decl: &DirectDeclarator,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    match direct_decl {
        DirectDeclarator::Ident(ident) => {
            print_branch("Ident", &ident.name, indent, is_last, &prefix);
        }
        DirectDeclarator::Paren(decl) => {
            print_branch("Parenthesized", "", indent, is_last, &prefix);
            visualize_declarator(decl, indent + 1, true, extend_prefix(&prefix, !is_last));
        }
        DirectDeclarator::Array { base, size } => {
            print_branch("Array", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Base", "", indent + 1, size.is_none(), &next_prefix);
            visualize_direct_declarator(
                base,
                indent + 2,
                true,
                extend_prefix(&next_prefix, size.is_some()),
            );

            if let Some(size) = size {
                print_branch("Size", "", indent + 1, true, &next_prefix);
                visualize_expr(size, indent + 2, true, extend_prefix(&next_prefix, false));
            }
        }
        DirectDeclarator::Func { base, params } => {
            print_branch("Function", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);

            print_branch("Base", "", indent + 1, params.is_none(), &next_prefix);
            visualize_direct_declarator(
                base,
                indent + 2,
                true,
                extend_prefix(&next_prefix, params.is_some()),
            );

            if let Some(params) = params {
                print_branch("Parameters", "", indent + 1, true, &next_prefix);
                visualize_param_list(params, indent + 2, true, extend_prefix(&next_prefix, false));
            }
        }
    }
}

fn visualize_param_list(param_list: &ParamList, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match param_list {
        ParamList::Void => {
            print_branch("Void", "", indent, is_last, &prefix);
        }
        ParamList::Params(params) => {
            if params.is_empty() {
                print_branch("(empty)", "", indent, is_last, &prefix);
            } else {
                for (i, param) in params.iter().enumerate() {
                    let is_last_param = i == params.len() - 1 && is_last;
                    print_branch(
                        "Parameter",
                        &format!("{:?} {}", param.ty, extract_function_name(&param.name)),
                        indent,
                        is_last_param,
                        &prefix,
                    );
                }
            }
        }
    }
}

fn visualize_initializer(
    initializer: &Initializer,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    match initializer {
        Initializer::Expr(expr) => {
            visualize_expr(expr, indent, is_last, prefix);
        }
        Initializer::List(list) => {
            if list.is_empty() {
                print_branch("InitList", "(empty)", indent, is_last, &prefix);
            } else {
                print_branch("InitList", "", indent, is_last, &prefix);
                for (i, init) in list.iter().enumerate() {
                    let last_init = i == list.len() - 1;
                    visualize_initializer(
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
            visualize_expr(
                &case_stmt.expr,
                indent + 2,
                true,
                extend_prefix(&next_prefix, !case_stmt.stmts.is_empty()),
            );

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

fn visualize_expr(expr: &Expr, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match expr {
        Expr::Num(n) => {
            print_branch("Number", &n.to_string(), indent, is_last, &prefix);
        }
        Expr::Char(c) => {
            print_branch("Character", &format!("'{}'", c), indent, is_last, &prefix);
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
