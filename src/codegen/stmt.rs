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
    // 初期化子がある場合は初期化
    if let Some(init_data) = init.r {
        match init_data {
            InitData::Expr(expr) => {
                cgs.outpus.push(StackCommand::Alloca(init.l.clone()));
                gen_expr(expr, cgs);
                cgs.outpus.push(StackCommand::Symbol(init.l));
                cgs.outpus.push(StackCommand::Store);
            }
            _ => {}
        }
    } else {
        cgs.outpus.push(StackCommand::Alloca(init.l.clone()));
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

fn declare_union(init: Union, _cgs: &mut CodeGenStatus) {
    println!(
        "{} = type {{[ {} x i8 ]}}",
        init.symbol.scope.ptr.upgrade().unwrap().borrow().symbols[init.ident.as_ref().unwrap()]
            .to_llvm_format(),
        init.size()
    );
}

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

fn initialize_variable(
    var_name: LLVMValue,
    init_data: InitData,
    var_type: &Type,
    cgs: &mut CodeGenStatus,
) {
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

fn r#return(ret: Return, cgs: &mut CodeGenStatus) {}

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
                let label_end = cgs.name_gen.slabel();
                let label_else = cgs.name_gen.slabel();
                gen_expr(*if_stmt.cond, cgs);
                cgs.outpus.push(StackCommand::JumpIfFalse(label_else));
                stmt(*if_stmt.then_branch, cgs);
                cgs.outpus.push(StackCommand::Jump(label_end));
                cgs.outpus.push(StackCommand::Label(label_else));
                stmt(*else_blach, cgs);
                cgs.outpus.push(StackCommand::Label(label_end));
            }
            None => {
                let label_end = cgs.name_gen.slabel();
                gen_expr(*if_stmt.cond, cgs);
                cgs.outpus.push(StackCommand::JumpIfFalse(label_end));
                stmt(*if_stmt.then_branch, cgs);
                cgs.outpus.push(StackCommand::Label(label_end));
            }
        }
    }

    pub fn r#while(while_stmt: While, cgs: &mut CodeGenStatus) {
        let label_start = cgs.name_gen.slabel();
        let label_end = cgs.name_gen.slabel();

        cgs.outpus.push(StackCommand::Label(label_start));
        gen_expr(*while_stmt.cond, cgs);
        cgs.outpus.push(StackCommand::JumpIfFalse(label_end));
        stmt(*while_stmt.body, cgs);
        cgs.outpus.push(StackCommand::Jump(label_start));
        cgs.outpus.push(StackCommand::Label(label_end));
    }

    pub fn r#do_while(do_while_stmt: DoWhile, cgs: &mut CodeGenStatus) {}

    pub fn r#for(for_stmt: For, cgs: &mut CodeGenStatus) {
        let label_start = cgs.name_gen.slabel();
        let label_step = cgs.name_gen.slabel();
        let label_end = cgs.name_gen.slabel();

        // 初期化式を実行
        if let Some(init) = for_stmt.init {
            gen_expr(*init, cgs);
        }

        // ループ開始
        cgs.outpus.push(StackCommand::Label(label_start));

        // 条件判定
        if let Some(cond) = for_stmt.cond {
            gen_expr(*cond, cgs);
            cgs.outpus.push(StackCommand::JumpIfFalse(label_end));
        }

        // ループ本体
        stmt(*for_stmt.body, cgs);

        // ステップ部分
        cgs.outpus.push(StackCommand::Label(label_step));
        if let Some(step) = for_stmt.step {
            gen_expr(*step, cgs);
        }

        // ループの先頭に戻る
        cgs.outpus.push(StackCommand::Jump(label_start));
        cgs.outpus.push(StackCommand::Label(label_end));
    }

    pub fn r#switch(switch_stmt: Switch, cgs: &mut CodeGenStatus) {}
}
