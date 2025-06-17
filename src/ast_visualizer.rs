use crate::symbols::{Control, Expr, FunctionDef, Program, Stmt, SwitchCase, TopLevel};

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
        }
    }
}

fn visualize_function_def(func: &FunctionDef, indent: usize, is_last: bool, prefix: Vec<bool>) {
    print_branch("FunctionDef", &func.name.name, indent, is_last, &prefix);

    let has_body = !func.body.statements.is_empty();
    let total_items = 2 + if has_body { 1 } else { 0 }; // ReturnType, Params, optionally Body

    print_branch(
        "ReturnType",
        &format!("{:?}", func.ret_type),
        indent + 1,
        total_items == 1,
        &extend_prefix(&prefix, !is_last),
    );

    match &func.params {
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
        Stmt::Decl(decl) => {
            print_branch("Decl", &decl.name.name, indent, is_last, &prefix);

            print_branch(
                "Type",
                &format!("{:?}", decl.ty),
                indent + 1,
                decl.init.is_none(),
                &extend_prefix(&prefix, !is_last),
            );

            if let Some(init) = &decl.init {
                print_branch(
                    "Init",
                    "",
                    indent + 1,
                    true,
                    &extend_prefix(&prefix, !is_last),
                );
                visualize_expr(
                    init,
                    indent + 2,
                    true,
                    extend_prefix(&extend_prefix(&prefix, !is_last), false),
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

                let has_cond = for_stmt.cond.is_some();
                let has_step = for_stmt.step.is_some();

                if let Some(init) = &for_stmt.init {
                    print_branch("Init", "", indent + 1, !has_cond && !has_step, &next_prefix);
                    visualize_expr(
                        init,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, has_cond || has_step),
                    );
                }
                if let Some(cond) = &for_stmt.cond {
                    print_branch("Cond", "", indent + 1, !has_step, &next_prefix);
                    visualize_expr(
                        cond,
                        indent + 2,
                        true,
                        extend_prefix(&next_prefix, has_step),
                    );
                }
                if let Some(step) = &for_stmt.step {
                    print_branch("Step", "", indent + 1, false, &next_prefix);
                    visualize_expr(step, indent + 2, true, extend_prefix(&next_prefix, true));
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
