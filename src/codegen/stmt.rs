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
        Stmt::ExprStmt(expr) => gen_expr(expr, cgs),
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
        DeclStmt::Typedef(_) => {}
        DeclStmt::Struct(this) => declare_struct(this, cgs),
        DeclStmt::Enum(this) => declare_enum(this, cgs),
        DeclStmt::Union(this) => declare_union(this, cgs),
    }
}

fn declare_variable(init: Init, cgs: &mut CodeGenStatus) {
    // メモリ確保と変数名を登録
    let var_type = init.l.get_type().unwrap();
    cgs.outputs.push(StackCommand::Alloc(var_type.clone()));
    cgs.outputs.push(StackCommand::Name(init.l.clone()));

    // 初期化子がある場合は初期化を実行
    if let Some(init_data) = init.r {
        initialize_variable(init.l.clone(), init_data, &var_type, cgs);
    }
}

impl Array {
    pub fn arry_list(&self) -> Vec<usize> {
        let this_len = self.length.as_ref().unwrap().consume_const() as usize;

        match &*self.array_of {
            Type::Array(inner) => {
                std::iter::once(self.length.as_ref().unwrap().consume_const() as usize)
                    .chain(inner.arry_list().into_iter())
                    .collect()
            }
            _ => vec![this_len],
        }
    }

    fn types(&self, b: usize) -> Type {
        if b == 0 {
            *self.array_of.clone()
        } else {
            self.array_of.as_array().unwrap().types(b - 1)
        }
    }
}

impl InitData {
    fn acsess(&self, list: Vec<usize>) -> InitData {
        if list.is_empty() {
            return self.clone();
        } else {
            match self {
                InitData::Compound(this) => this[*list.first().unwrap()].acsess(list[1..].to_vec()),
                _ => return self.clone(),
            }
        }
    }
}

fn initialize_variable(
    object: Symbol,
    init_data: InitData,
    var_type: &Type,
    cgs: &mut CodeGenStatus,
) {
    match init_data.clone() {
        InitData::Expr(typed_expr) => {
            // 式の初期化: 値を評価してスタックに乗せる
            gen_expr(typed_expr, cgs);
            cgs.outputs.push(StackCommand::Symbol(object.clone()));
            cgs.outputs.push(StackCommand::AcsessUseLa);
            cgs.outputs.push(StackCommand::Store);
        }
        InitData::Compound(_) => {
            // 複合初期化子 {1, 2, 3} または {.a = 1, .b = 2}
            match var_type {
                Type::Array(arr) => {
                    let combos =
                        arr.arry_list()
                            .iter()
                            .fold(vec![vec![]].into_iter(), |acc, &max| {
                                acc.flat_map(move |prefix| {
                                    (0..max).map(move |i| {
                                        let mut new_prefix = prefix.clone();
                                        new_prefix.push(i);
                                        new_prefix
                                    })
                                })
                                .collect::<Vec<_>>()
                                .into_iter()
                            });

                    for i in combos {
                        let tmp = i.clone();
                        match init_data.clone().acsess(i) {
                            InitData::Compound(_) => panic!(),
                            InitData::Expr(this) => gen_expr(this, cgs),
                        }

                        cgs.outputs.push(StackCommand::Symbol(object.clone()));
                        cgs.outputs.push(StackCommand::AcsessUseLa);

                        for i in (0..tmp.len()).rev() {
                            let ty = arr.types(tmp.len() - i - 1);
                            let offset = tmp[tmp.len() - i - 1];
                            cgs.outputs.push(StackCommand::Push(TypedExpr::new(
                                Type::Int,
                                SemaExpr::NumInt(offset),
                            )));
                            cgs.outputs.push(StackCommand::IndexAccess(ty));
                        }
                        cgs.outputs.push(StackCommand::Store);
                    }
                }
                Type::Struct(stru) => {
                    // 構造体の初期化
                }
                Type::Union(_) => {
                    panic!("共用体の複合初期化は未対応");
                }
                _ => {
                    panic!("複合初期化子が使用できない型です");
                }
            }
        }
    }
}

fn declare_struct(init: Struct, _cgs: &mut CodeGenStatus) {
    // 変数を割り当て
    println!(
        "{} = type {{{}}}",
        init.symbol.get_type().unwrap().to_llvm_format(),
        init.member
            .iter()
            .map(|x| x.get_type().unwrap().to_llvm_format())
            .collect::<Vec<String>>()
            .join(",")
    );
}

fn declare_union(init: Union, _cgs: &mut CodeGenStatus) {}

fn declare_enum(init: Enum, cgs: &mut CodeGenStatus) {
    let mut start = 0;

    for i in 0..init.variants.len() {
        if let Some(num) = init.variants[i].value {
            start = num;
        }

        let name = cgs.name_gen.global_const().to_string();
        println!(
            "{} = constant {} {}",
            name,
            Type::Int.to_llvm_format(),
            start
        );
        cgs.register_variable(init.variants[i].symbol.clone(), name);
        start += 1
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

fn r#break(cgs: &mut CodeGenStatus) {}

fn r#continue(cgs: &mut CodeGenStatus) {}

fn r#return(ret: Return, cgs: &mut CodeGenStatus) {
    if let Some(expr) = ret.value {
        gen_expr(*expr.clone(), cgs);
        cgs.outputs.push(StackCommand::Return);
    }

    cgs.outputs.push(StackCommand::Goto(cgs.func_end.unwrap()));
    cgs.outputs.push(StackCommand::Label(cgs.name_gen.slabel())); //未到達空間回避
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
        match if_stmt.else_branch {
            Some(else_blach) => {
                let label_then = cgs.name_gen.slabel();
                let label_else = cgs.name_gen.slabel();
                let label_end = cgs.name_gen.slabel();
                gen_expr(*if_stmt.cond, cgs);

                cgs.outputs
                    .push(StackCommand::Branch(label_then, label_else));

                cgs.outputs.push(label_then.into());
                stmt(*if_stmt.then_branch, cgs);
                cgs.outputs.push(StackCommand::Goto(label_end));

                cgs.outputs.push(label_else.into());
                stmt(*else_blach, cgs);
                cgs.outputs.push(StackCommand::Goto(label_end));

                cgs.outputs.push(label_end.into());
            }
            None => {
                let label_then = cgs.name_gen.slabel();
                let label_end = cgs.name_gen.slabel();
                gen_expr(*if_stmt.cond, cgs);
                cgs.outputs
                    .push(StackCommand::Branch(label_then, label_end));
                cgs.outputs.push(label_then.into());
                stmt(*if_stmt.then_branch, cgs);
                cgs.outputs.push(StackCommand::Goto(label_end));
                cgs.outputs.push(label_end.into());
            }
        }
    }

    pub fn r#while(while_stmt: While, cgs: &mut CodeGenStatus) {
        let label_start = cgs.name_gen.slabel();
        let label_body = cgs.name_gen.slabel();
        let label_end = cgs.name_gen.slabel();

        // ループの先頭ラベル
        cgs.outputs.push(StackCommand::Label(label_start));

        // 条件式を評価
        gen_expr(*while_stmt.cond, cgs);

        // 条件が真なら body へ、偽なら end へ
        cgs.outputs
            .push(StackCommand::Branch(label_body, label_end));

        // ループ本体
        cgs.outputs.push(StackCommand::Label(label_body));
        stmt(*while_stmt.body, cgs);

        // 本体実行後、再び条件評価へ戻る
        cgs.outputs.push(StackCommand::Goto(label_start));

        // 終了ラベル
        cgs.outputs.push(StackCommand::Label(label_end));
    }

    pub fn r#do_while(do_while_stmt: DoWhile, cgs: &mut CodeGenStatus) {}

    pub fn r#for(for_stmt: For, cgs: &mut CodeGenStatus) {
        let label_start = cgs.name_gen.slabel();
        let label_body = cgs.name_gen.slabel();
        let label_step = cgs.name_gen.slabel();
        let label_end = cgs.name_gen.slabel();

        // 初期化式
        if let Some(init) = for_stmt.init {
            gen_expr(*init, cgs);
        }

        // 条件判定へ
        cgs.outputs.push(label_start.into());

        if let Some(cond) = for_stmt.cond {
            gen_expr(*cond, cgs);
            // 条件が真なら本体へ、偽なら終了
            cgs.outputs
                .push(StackCommand::Branch(label_body, label_end));
        } else {
            // 条件なしの場合は無限ループ
            cgs.outputs.push(StackCommand::Goto(label_body));
        }

        // ループ本体
        cgs.outputs.push(label_body.into());
        stmt(*for_stmt.body, cgs);
        cgs.outputs.push(StackCommand::Goto(label_step));

        // ステップ（後処理）
        cgs.outputs.push(label_step.into());
        if let Some(step) = for_stmt.step {
            gen_expr(*step, cgs);
        }

        // 再び条件判定へ
        cgs.outputs.push(StackCommand::Goto(label_start));

        // ループ終了
        cgs.outputs.push(label_end.into());
    }

    pub fn r#switch(switch_stmt: Switch, cgs: &mut CodeGenStatus) {}
}
