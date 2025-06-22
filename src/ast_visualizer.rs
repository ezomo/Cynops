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

fn visualize_function_proto(
    proto: &FunctionProto,
    indent: usize,
    is_last: bool,
    prefix: Vec<bool>,
) {
    print_branch(
        "FunctionProto",
        &proto.sig.name.name,
        indent,
        is_last,
        &prefix,
    );

    let has_params = match &proto.sig.params {
        ParamList::Void => false,
        ParamList::Params(params) => !params.is_empty(),
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
        if let ParamList::Params(params) = &proto.sig.params {
            for (i, param) in params.iter().enumerate() {
                let is_last_param = i == params.len() - 1;
                print_branch(
                    "Param",
                    &format!("{:?} {}", param.ty, param.name.name),
                    indent + 2,
                    is_last_param,
                    &extend_prefix(&extend_prefix(&prefix, !is_last), false),
                );
            }
        }
    } else {
        print_branch(
            "Params",
            "(empty)",
            indent + 1,
            true,
            &extend_prefix(&prefix, !is_last),
        );
    }
}

fn visualize_function_def(func: &FunctionDef, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch("FunctionDef", &func.sig.name.name, indent, is_last, &prefix);

    let has_body = !func.body.statements.is_empty();
    let total_items = 2 + if has_body { 1 } else { 0 }; // ReturnType, Params, optionally Body

    print_branch(
        "ReturnType",
        &format!("{:?}", func.sig.ret_type),
        indent + 1,
        total_items == 1,
        &extend_prefix(&prefix, !is_last),
    );

    match &func.sig.params {
        crate::symbols::ParamList::Void => {
            print_branch(
                "Params",
                "void",
                indent + 1,
                !has_body,
                &extend_prefix(&prefix, !is_last),
            );
        }
        crate::symbols::ParamList::Params(params) => {
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
                        &format!("{:?} {}", param.ty, param.name.name),
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
            print_branch("Return", "", indent, is_last, &prefix);
            if let Some(expr) = &ret.value {
                visualize_expr(expr, indent + 1, true, extend_prefix(&prefix, !is_last));
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
                print_branch("Cond", "", indent + 1, false, &next_prefix);
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
                print_branch("Cond", "", indent + 1, false, &next_prefix);
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
                    print_branch("Cond", "", indent + 1, remaining_items == 0, &next_prefix);
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
                print_branch("Cond", "", indent + 1, true, &next_prefix);
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
                print_branch("Cond", "", indent + 1, false, &next_prefix);
                visualize_expr(
                    &switch_stmt.cond,
                    indent + 2,
                    true,
                    extend_prefix(&next_prefix, true),
                );
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
        },
        Stmt::Block(block) => {
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
            print_branch(
                "Stmt",
                "",
                indent + 1,
                true,
                &extend_prefix(&prefix, !is_last),
            );
            visualize_stmt(
                &label.stmt,
                indent + 2,
                true,
                extend_prefix(&extend_prefix(&prefix, !is_last), false),
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
    print_branch("InitDeclarator", "", indent, is_last, &prefix);
    let next_prefix = extend_prefix(&prefix, !is_last);

    print_branch(
        "Declarator",
        "",
        indent + 1,
        init_decl.init.is_none(),
        &next_prefix,
    );
    visualize_declarator(
        &init_decl.declarator,
        indent + 2,
        true,
        extend_prefix(&next_prefix, init_decl.init.is_some()),
    );

    if let Some(init) = &init_decl.init {
        print_branch("Init", "", indent + 1, true, &next_prefix);
        visualize_initializer(init, indent + 2, true, extend_prefix(&next_prefix, false));
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
            print_branch("Paren", "", indent, is_last, &prefix);
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
            print_branch("Func", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);
            print_branch("Base", "", indent + 1, params.is_none(), &next_prefix);
            visualize_direct_declarator(
                base,
                indent + 2,
                true,
                extend_prefix(&next_prefix, params.is_some()),
            );
            if let Some(params) = params {
                print_branch("Params", "", indent + 1, true, &next_prefix);
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
            for (i, param) in params.iter().enumerate() {
                let is_last_param = i == params.len() - 1;
                print_branch(
                    "Param",
                    &format!("{:?} {}", param.ty, param.name.name),
                    indent,
                    is_last_param,
                    &prefix,
                );
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

fn visualize_switch_case(case: &SwitchCase, indent: usize, is_last: bool, prefix: Vec<bool>) {
    match case {
        SwitchCase::Case(case_stmt) => {
            print_branch("Case", "", indent, is_last, &prefix);
            let next_prefix = extend_prefix(&prefix, !is_last);
            print_branch(
                "Expr",
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
                print_branch("Stmts", "", indent + 1, true, &next_prefix);
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
            print_branch("Default", "", indent, is_last, &prefix);
            if !default_case.stmts.is_empty() {
                let next_prefix = extend_prefix(&prefix, !is_last);
                print_branch("Stmts", "", indent + 1, true, &next_prefix);
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
            print_branch("Num", &n.to_string(), indent, is_last, &prefix);
        }
        Expr::Char(c) => {
            print_branch("Char", &format!("'{}'", c), indent, is_last, &prefix);
        }
        Expr::Ident(name) => {
            print_branch("Ident", &name.name, indent, is_last, &prefix);
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
            print_branch("LHS", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &binary.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );
            print_branch("RHS", "", indent + 1, true, &new_prefix);
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
            visualize_expr(
                &unary.expr,
                indent + 1,
                true,
                extend_prefix(&prefix, !is_last),
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
            visualize_expr(
                &postfix.expr,
                indent + 1,
                true,
                extend_prefix(&prefix, !is_last),
            );
        }
        Expr::Call(call) => {
            print_branch("Call", &call.func.name, indent, is_last, &prefix);
            if !call.args.is_empty() {
                print_branch(
                    "Args",
                    "",
                    indent + 1,
                    true,
                    &extend_prefix(&prefix, !is_last),
                );
                for (i, arg) in call.args.iter().enumerate() {
                    let last = i == call.args.len() - 1;
                    visualize_expr(
                        arg,
                        indent + 2,
                        last,
                        extend_prefix(&extend_prefix(&prefix, !is_last), false),
                    );
                }
            }
        }
        Expr::Assign(assign) => {
            print_branch(
                "Assign",
                &format!("{:?}", assign.op),
                indent,
                is_last,
                &prefix,
            );
            let new_prefix = extend_prefix(&prefix, !is_last);
            print_branch("LHS", "", indent + 1, false, &new_prefix);
            visualize_expr(
                &assign.lhs,
                indent + 2,
                true,
                extend_prefix(&new_prefix, true),
            );
            print_branch("RHS", "", indent + 1, true, &new_prefix);
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
            print_branch("Cond", "", indent + 1, false, &new_prefix);
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
    }
}

fn print_branch(label: &str, value: &str, _indent: usize, is_last: bool, prefix: &[bool]) {
    for &p in prefix {
        if p {
            print!("│   ");
        } else {
            print!("    ");
        }
    }

    if is_last {
        print!("└── ");
    } else {
        print!("├── ");
    }

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
