use super::*;
use crate::sema::ast::*;

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
        Stmt::ExprStmt(expr) => {
            let _ = new_load(gen_expr, expr, cgs);
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
    let var_name = cgs.name_gen.register();
    let llvm_type = &init.r.ty.to_llvm_format();

    // 変数を割り当て
    println!("{} = alloca {}", var_name.to_string(), llvm_type);

    // 変数名をマップに登録
    cgs.register_variable(init.r.sympl, var_name.clone().to_string());

    // 初期化子がある場合は初期化
    if let Some(init_data) = init.l {
        initialize_variable(&var_name.to_string(), init_data, &init.r.ty, cgs);
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
            let value = load(
                &typed_expr.r#type.clone(),
                new_load(gen_expr, typed_expr, cgs),
                cgs,
            );
            let llvm_type = var_type.to_llvm_format();
            println!(
                "  store {} {}, {}* {}",
                llvm_type,
                value.to_string(),
                llvm_type,
                var_name
            );
        }
        InitData::Compound(compound_list) => {
            match var_type {
                Type::Array(arr) => {
                    // 配列の初期化 {1, 2, 3}
                    for (index, element) in compound_list.into_iter().enumerate() {
                        let element_ptr = cgs.name_gen.variable();
                        let array_type = format!(
                            "[{} x {}]",
                            arr.length.clone().unwrap().consume_const(),
                            &arr.array_of.to_llvm_format()
                        );
                        println!(
                            "  {} = getelementptr inbounds {}, {}* {}, i64 0, i64 {}",
                            element_ptr.to_string(),
                            array_type,
                            array_type,
                            var_name,
                            index
                        );

                        initialize_variable(&element_ptr.to_string(), element, &arr.array_of, cgs);
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
        println!("  br label %{}", break_label.to_string());
    } else {
        panic!("break statement outside of loop");
    }
}

fn r#continue(cgs: &mut CodeGenStatus) {
    if let Some(continue_label) = cgs.current_continue_label() {
        println!("  br label %{}", continue_label.to_string());
    } else {
        panic!("continue statement outside of loop");
    }
}

fn r#return(ret: Return, cgs: &mut CodeGenStatus) {
    if let Some(expr) = ret.value {
        let ty = expr.r#type.clone();
        let val = new_load(gen_expr, *expr, cgs);

        // return値をreturn_value_ptrに保存
        if let Some(ref return_ptr) = cgs.return_value_ptr {
            println!(
                "store {} {}, {}* {}",
                ty.to_llvm_format(),
                val.to_string(),
                ty.to_llvm_format(),
                return_ptr
            );
        }
    }

    // return_labelにジャンプ
    if let Some(ref return_label) = cgs.return_label {
        println!("br label %{}", return_label.to_string());
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
        let then_label = cgs.name_gen.label();
        let else_label = cgs.name_gen.label();
        let end_label = cgs.name_gen.label();

        // 条件の評価（TypedExprなのでtodo!()）
        let cond_result = new_load(gen_expr, *if_stmt.cond, cgs).i64toi1(cgs); // todo!()で条件式を評価した結果

        // 条件による分岐
        if if_stmt.else_branch.is_some() {
            println!(
                "  br i1 {}, label %{}, label %{}",
                cond_result.to_string(),
                then_label.to_string(),
                else_label.to_string()
            );
        } else {
            println!(
                "  br i1 {}, label %{}, label %{}",
                cond_result.to_string(),
                then_label.to_string(),
                end_label.to_string()
            );
        }

        // then ブロック
        println!("{}:", then_label.to_string());
        stmt(*if_stmt.then_branch, cgs);
        println!("  br label %{}", end_label.to_string());

        // else ブロック（存在する場合）
        if let Some(else_branch) = if_stmt.else_branch {
            println!("{}:", else_label.to_string());
            stmt(*else_branch, cgs);
            println!("  br label %{}", end_label.to_string());
        }

        // 終了ラベル
        println!("{}:", end_label.to_string());
    }

    pub fn r#while(while_stmt: While, cgs: &mut CodeGenStatus) {
        let cond_label = cgs.name_gen.label();
        let body_label = cgs.name_gen.label();
        let end_label = cgs.name_gen.label();

        // ループラベルをプッシュ
        cgs.push_loop_labels(end_label.clone(), cond_label.clone());

        // 条件の評価へジャンプ
        println!("  br label %{}", cond_label.to_string());

        // 条件評価ラベル
        println!("{}:", cond_label.to_string());
        let cond_result = new_load(gen_expr, *while_stmt.cond, cgs).i64toi1(cgs); // todo!()で条件式を評価した結果
        println!(
            "  br i1 {}, label %{}, label %{}",
            cond_result.to_string(),
            body_label.to_string(),
            end_label.to_string()
        );

        // 本体ラベル
        println!("{}:", body_label.to_string());
        stmt(*while_stmt.body, cgs);
        println!("  br label %{}", cond_label.to_string());

        // 終了ラベル
        println!("{}:", end_label.to_string());

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#do_while(do_while_stmt: DoWhile, cgs: &mut CodeGenStatus) {
        let body_label = cgs.name_gen.label();
        let cond_label = cgs.name_gen.label();
        let end_label = cgs.name_gen.label();

        // ループラベルをプッシュ
        cgs.push_loop_labels(end_label.clone(), cond_label.clone());

        // 本体へジャンプ
        println!("  br label %{}", body_label.to_string());

        // 本体ラベル
        println!("{}:", body_label.to_string());
        stmt(*do_while_stmt.body, cgs);
        println!("  br label %{}", cond_label.to_string());

        // 条件評価ラベル
        println!("{}:", cond_label.to_string());
        let cond_result = new_load(gen_expr, *do_while_stmt.cond, cgs).i64toi1(cgs); // todo!()で条件式を評価した結果
        println!(
            "  br i1 {}, label %{}, label %{}",
            cond_result.to_string(),
            body_label.to_string(),
            end_label.to_string()
        );

        // 終了ラベル
        println!("{}:", end_label.to_string());

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#for(for_stmt: For, cgs: &mut CodeGenStatus) {
        let cond_label = cgs.name_gen.label();
        let body_label = cgs.name_gen.label();
        let step_label = cgs.name_gen.label();
        let end_label = cgs.name_gen.label();

        // ループラベルをプッシュ（continueはstepラベルへ）
        cgs.push_loop_labels(end_label.clone(), step_label.clone());

        // 初期化
        if let Some(_init) = for_stmt.init {
            let _ = new_load(gen_expr, *_init, cgs);
        }

        // 条件の評価へジャンプ
        println!("  br label %{}", cond_label.to_string());

        // 条件評価ラベル
        println!("{}:", cond_label.to_string());
        if let Some(_cond) = for_stmt.cond {
            let cond_result = new_load(gen_expr, *_cond, cgs).i64toi1(cgs); // todo!()で条件式を評価した結果
            println!(
                "  br i1 {}, label %{}, label %{}",
                cond_result.to_string(),
                body_label.to_string(),
                end_label.to_string()
            );
        } else {
            // 条件なし（無限ループ）
            println!("  br label %{}", body_label.to_string());
        }

        // 本体ラベル
        println!("{}:", body_label.to_string());
        stmt(*for_stmt.body, cgs);
        println!("  br label %{}", step_label.to_string());

        // ステップラベル
        println!("{}:", step_label.to_string());
        if let Some(_step) = for_stmt.step {
            new_load(gen_expr, *_step, cgs);
        }
        println!("  br label %{}", cond_label.to_string());

        // 終了ラベル
        println!("{}:", end_label.to_string());

        // ループラベルをポップ
        cgs.pop_loop_labels();
    }

    pub fn r#switch(switch_stmt: Switch, cgs: &mut CodeGenStatus) {
        let end_label = cgs.name_gen.label();
        let default_label = cgs.name_gen.label();

        // breakラベルをプッシュ（switchではcontinueは使用不可なので空文字列）
        cgs.break_labels.push(end_label.clone());

        let cond_result = new_load(gen_expr, *switch_stmt.cond, cgs);

        // switchの開始
        print!(
            "  switch i64 {}, label %{} [",
            cond_result.to_string(),
            default_label.to_string()
        );

        let mut case_labels = Vec::new();
        let mut has_default = false;

        // ケースラベルの生成
        for case in switch_stmt.cases.iter() {
            match case {
                SwitchCase::Case(case_stmt) => {
                    let case_label = cgs.name_gen.label();
                    case_labels.push((case_label.clone(), case));
                    let case_value = case_stmt.const_expr.consume_const(); // todo!()でcase値を評価
                    print!(
                        "\n    i64 {}, label %{}",
                        case_value,
                        case_label.to_string()
                    );
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
                println!("{}:", label.to_string());
                for stmt in &case_stmt.stmts {
                    super::stmt(*stmt.clone(), cgs);
                }
                // break文が無い場合は次のcaseへfall through
                if i < case_labels.len() - 1 {
                    println!("  br label %{}", case_labels[i + 1].0.to_string());
                }
            }
        }

        // defaultケースの処理
        println!("{}:", default_label.to_string());
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
        println!("  br label %{}", end_label.to_string());

        // 終了ラベル
        println!("{}:", end_label.to_string());

        // breakラベルをポップ
        cgs.break_labels.pop();
    }
}
