use super::*;
use crate::ast::*;

pub fn stmt(stmt: Stmt, cgs: &mut CodeGenStatus) {
    match stmt {
        Stmt::Block(block) => self::block(block, cgs),
        Stmt::DeclStmt(declstmt) => self::declstmt(declstmt, cgs),
        Stmt::Control(control) => self::control(control, cgs),
        Stmt::Break => r#break(cgs),
        Stmt::Continue => r#continue(cgs),
        Stmt::Return(ret) => r#return(ret, cgs),
        Stmt::Goto(goto) => self::goto(goto, cgs),
        Stmt::Label(label) => self::label(label, cgs),
        Stmt::TypedExprStmt(expr) => {
            let _ = gen_typed_expr(expr, cgs);
        }
    }
}

pub fn block(block: Block, cgs: &mut CodeGenStatus) {
    for _stmt in block.into_vec() {
        stmt(*_stmt, cgs);
    }
}

fn declstmt(declstmt: DeclStmt, cgs: &mut CodeGenStatus) {
    match declstmt {
        DeclStmt::InitVec(inits) => {
            for init in inits {
                declare_variable(init, cgs);
            }
        }
        _ => {
            // Struct, Union, Enum, Typedef は今回は対象外
            todo!("構造体、共用体、列挙型、typedef は未対応")
        }
    }
}

fn declare_variable(init: Init, cgs: &mut CodeGenStatus) {
    let var_name = cgs.name_gen.next_with_prefix("var");
    let llvm_type = &init.r.ty.get_llvm_type();

    // 変数を割り当て
    println!("  %{} = alloca {}", var_name, llvm_type);

    // 変数名をマップに登録
    cgs.variables.insert(init.r.ident.clone(), var_name.clone());

    // 初期化子がある場合は初期化
    if let Some(init_data) = init.l {
        initialize_variable(&var_name, init_data, &init.r.ty, cgs);
    }
}

fn initialize_variable(
    var_name: &str,
    init_data: InitData,
    var_type: &Type,
    cgs: &mut CodeGenStatus,
) {
    match init_data {
        InitData::Expr(typed_expr) => {
            // 単純な式による初期化
            let value = gen_typed_expr(typed_expr, cgs);
            let llvm_type = var_type.get_llvm_type();
            println!("  store {} %{}, ptr %{}", llvm_type, value, var_name);
        }
        InitData::Compound(compound_list) => {
            match var_type {
                Type::Array(arr) => {
                    // 配列の初期化 {1, 2, 3}
                    for (index, element) in compound_list.into_iter().enumerate() {
                        let element_ptr = cgs.name_gen.next();
                        let array_type =
                            format!("[{} x {}]", arr.length, &arr.array_of.get_llvm_type());
                        println!(
                            "  %{} = getelementptr inbounds {}, ptr %{}, i64 0, i64 {}",
                            element_ptr, array_type, var_name, index
                        );

                        initialize_variable(&element_ptr, element, &arr.array_of, cgs);
                    }
                }
                _ => {
                    todo!("構造体・共用体の複合初期化は未対応")
                }
            }
        }
    }
}

fn control(control: Control, cgs: &mut CodeGenStatus) {
    match control {
        Control::If(if_stmt) => controls::r#if(if_stmt, cgs),
        Control::While(while_stmt) => controls::r#while(while_stmt, cgs),
        Control::DoWhile(do_while_stmt) => controls::r#do_while(do_while_stmt, cgs),
        Control::For(for_stmt) => controls::r#for(for_stmt, cgs),
        Control::Switch(switch_stmt) => controls::r#switch(switch_stmt, cgs),
    }
}

fn r#break(cgs: &mut CodeGenStatus) {
    if let Some(break_label) = cgs.current_break_label() {
        println!("  br label %{}", break_label);
    } else {
        panic!("break statement outside of loop");
    }
}

fn r#continue(cgs: &mut CodeGenStatus) {
    if let Some(continue_label) = cgs.current_continue_label() {
        println!("  br label %{}", continue_label);
    } else {
        panic!("continue statement outside of loop");
    }
}

fn r#return(ret: Return, cgs: &mut CodeGenStatus) {
    let rhs = if let Some(value) = ret.value {
        gen_typed_expr(*value, cgs)
    } else {
        // voidの場合は0を返す
        let name = cgs.name_gen.next();
        println!("%{} = add i64 0, 0", name);
        name
    };

    // return値をreturn_value_ptrに保存
    if let Some(ref return_ptr) = cgs.return_value_ptr {
        println!("store i64 %{}, ptr %{}", rhs, return_ptr);
    }

    // return_labelにジャンプ
    if let Some(ref return_label) = cgs.return_label {
        println!("br label %{}", return_label);
    }
}

fn goto(goto: Goto, _cgs: &mut CodeGenStatus) {
    println!("  br label %{}", goto.label.get_name());
}

fn label(label: Label, cgs: &mut CodeGenStatus) {
    println!("br label %{}", label.name.get_name());
    println!("{}:", label.name.get_name());
    stmt(*label.stmt, cgs);
}

mod controls {
    use super::*;

    pub fn r#if(if_stmt: If, cgs: &mut CodeGenStatus) {
        let then_label = cgs.next_label("then");
        let else_label = cgs.next_label("else");
        let end_label = cgs.next_label("end");

        // 条件の評価（TypedExprなのでtodo!()）
        let cond_result = gen_typed_expr(*if_stmt.cond, cgs); // todo!()で条件式を評価した結果

        // 条件による分岐
        if if_stmt.else_branch.is_some() {
            println!(
                "  br i1 %{}, label %{}, label %{}",
                cond_result, then_label, else_label
            );
        } else {
            println!(
                "  br i1 %{}, label %{}, label %{}",
                cond_result, then_label, end_label
            );
        }

        // then ブロック
        println!("{}:", then_label);
        stmt(*if_stmt.then_branch, cgs);
        println!("  br label %{}", end_label);

        // else ブロック（存在する場合）
        if let Some(else_branch) = if_stmt.else_branch {
            println!("{}:", else_label);
            stmt(*else_branch, cgs);
            println!("  br label %{}", end_label);
        }

        // 終了ラベル
        println!("{}:", end_label);
    }

    pub fn r#while(while_stmt: While, cgs: &mut CodeGenStatus) {
        let cond_label = cgs.next_label("while_cond");
        let body_label = cgs.next_label("while_body");
        let end_label = cgs.next_label("while_end");

        // ループラベルをプッシュ
        cgs.push_loop_labels(end_label.clone(), cond_label.clone());

        // 条件の評価へジャンプ
        println!("  br label %{}", cond_label);

        // 条件評価ラベル
        println!("{}:", cond_label);
        let cond_result = gen_typed_expr(*while_stmt.cond, cgs); // todo!()で条件式を評価した結果
        println!(
            "  br i1 %{}, label %{}, label %{}",
            cond_result, body_label, end_label
        );

        // 本体ラベル
        println!("{}:", body_label);
        stmt(*while_stmt.body, cgs);
        println!("  br label %{}", cond_label);

        // 終了ラベル
        println!("{}:", end_label);

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#do_while(do_while_stmt: DoWhile, cgs: &mut CodeGenStatus) {
        let body_label = cgs.next_label("do_body");
        let cond_label = cgs.next_label("do_cond");
        let end_label = cgs.next_label("do_end");

        // ループラベルをプッシュ
        cgs.push_loop_labels(end_label.clone(), cond_label.clone());

        // 本体へジャンプ
        println!("  br label %{}", body_label);

        // 本体ラベル
        println!("{}:", body_label);
        stmt(*do_while_stmt.body, cgs);
        println!("  br label %{}", cond_label);

        // 条件評価ラベル
        println!("{}:", cond_label);
        let cond_result = gen_typed_expr(*do_while_stmt.cond, cgs); // todo!()で条件式を評価した結果
        println!(
            "  br i1 %{}, label %{}, label %{}",
            cond_result, body_label, end_label
        );

        // 終了ラベル
        println!("{}:", end_label);

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#for(for_stmt: For, cgs: &mut CodeGenStatus) {
        let cond_label = cgs.next_label("for_cond");
        let body_label = cgs.next_label("for_body");
        let step_label = cgs.next_label("for_step");
        let end_label = cgs.next_label("for_end");

        // ループラベルをプッシュ（continueはstepラベルへ）
        cgs.push_loop_labels(end_label.clone(), step_label.clone());

        // 初期化
        if let Some(_init) = for_stmt.init {
            let _ = gen_typed_expr(*_init, cgs);
        }

        // 条件の評価へジャンプ
        println!("  br label %{}", cond_label);

        // 条件評価ラベル
        println!("{}:", cond_label);
        if let Some(_cond) = for_stmt.cond {
            let cond_result = gen_typed_expr(*_cond, cgs); // todo!()で条件式を評価した結果
            println!(
                "  br i1 %{}, label %{}, label %{}",
                cond_result, body_label, end_label
            );
        } else {
            // 条件なし（無限ループ）
            println!("  br label %{}", body_label);
        }

        // 本体ラベル
        println!("{}:", body_label);
        stmt(*for_stmt.body, cgs);
        println!("  br label %{}", step_label);

        // ステップラベル
        println!("{}:", step_label);
        if let Some(_step) = for_stmt.step {
            gen_typed_expr(*_step, cgs);
        }
        println!("  br label %{}", cond_label);

        // 終了ラベル
        println!("{}:", end_label);

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#switch(switch_stmt: Switch, cgs: &mut CodeGenStatus) {
        let end_label = cgs.next_label("switch_end");
        let default_label = cgs.next_label("switch_default");

        // breakラベルをプッシュ（switchではcontinueは使用不可なので空文字列）
        cgs.break_labels.push(end_label.clone());

        let cond_result = gen_typed_expr(*switch_stmt.cond, cgs);

        // switchの開始
        print!("  switch i64 %{}, label %{} [", cond_result, default_label);

        let mut case_labels = Vec::new();
        let mut has_default = false;

        // ケースラベルの生成
        for (i, case) in switch_stmt.cases.iter().enumerate() {
            match case {
                SwitchCase::Case(case_stmt) => {
                    let case_label = cgs.next_label(&format!("case_{}", i));
                    case_labels.push((case_label.clone(), case));
                    let case_value = case_stmt.const_expr.to_string(); // todo!()でcase値を評価
                    print!("\n    i64 {}, label %{}", case_value, case_label);
                }
                SwitchCase::Default(_) => {
                    has_default = true;
                }
            }
        }
        println!("\n  ]");

        // 各caseの処理
        for i in 0..case_labels.len() {
            let (label, case) = &case_labels[i];
            if let SwitchCase::Case(case_stmt) = case {
                println!("{}:", label);
                for stmt in &case_stmt.stmts {
                    super::stmt(*stmt.clone(), cgs);
                }
                // break文が無い場合は次のcaseへfall through
                if i < case_labels.len() - 1 {
                    println!("  br label %{}", case_labels[i + 1].0);
                }
            }
        }

        // defaultケースの処理
        println!("{}:", default_label);
        if has_default {
            for case in &switch_stmt.cases {
                if let SwitchCase::Default(default_case) = case {
                    for stmt in &default_case.stmts {
                        super::stmt(*stmt.clone(), cgs);
                    }
                    break;
                }
            }
        }
        println!("  br label %{}", end_label);

        // 終了ラベル
        println!("{}:", end_label);

        // breakラベルをポップ
        cgs.break_labels.pop();
    }
}
