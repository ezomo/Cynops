use super::*;
use crate::{sema::ast::*, visualize::OneLine};

fn function_def(function: FunctionDef, cgs: &mut CodeGenStatus) {
    let args: Vec<(Symbol, Type)> = (0..function.param_names.len())
        .map(|i| {
            (
                function.param_names[i].clone(),
                function
                    .sig
                    .symbol
                    .get_type()
                    .unwrap()
                    .as_func()
                    .unwrap()
                    .params[i]
                    .clone(),
            )
        })
        .collect();

    println!(
        "define {} {}({}) {{",
        function
            .sig
            .symbol
            .get_type()
            .unwrap()
            .as_func()
            .unwrap()
            .return_type
            .to_llvm_format(),
        function.sig.symbol.ident.get_fnc_name(),
        args.iter()
            .map(|x| format!("{} %{}", x.1.to_llvm_format(), x.0.ident.get_name()))
            .collect::<Vec<_>>()
            .join(", "),
    );

    {
        for (ident, ty) in &args {
            let ptr = cgs.name_gen.register();
            println!("{} = alloca {}", ptr.to_string(), ty.to_llvm_format());
            cgs.register_variable(ident.clone(), ptr.to_string());
        }
        for (ident, ty) in &args {
            println!(
                "store {} %{}, {}* {}",
                ty.to_llvm_format(),
                ident.ident.get_name(),
                ty.to_llvm_format(),
                cgs.get_variable(ident).unwrap()
            );
        }
    }

    // return用の変数とラベルを設定

    let return_type = function
        .sig
        .symbol
        .get_type()
        .unwrap()
        .as_func()
        .unwrap()
        .return_type
        .clone();
    let return_label = cgs.name_gen.label();
    let return_ptr = cgs.name_gen.register();

    if !return_type.is_void() {
        println!(
            "{} = alloca {}",
            return_ptr.to_string(),
            return_type.to_llvm_format()
        );
        cgs.return_value_ptr = Some(return_ptr.clone().to_string());
    }

    cgs.return_label = Some(return_label.clone());

    // 引数の処理

    // 関数本体の処理
    {
        for stmt in function.body.into_vec() {
            super::stmt::stmt(*stmt, cgs);
        }
    }

    // 常にreturn_labelにジャンプ（return文がない場合のため）
    println!("br label %{}", return_label.to_string());

    // return_labelとreturn処理
    println!("{}:", return_label.to_string());

    if !return_type.is_void() {
        println!(
            "%val = load {}, ptr {}",
            return_type.to_llvm_format(),
            return_ptr.to_string()
        );
        println!("ret {} %val", return_type.to_llvm_format());
    } else {
        println!("ret void");
    }

    println!("}}");

    // 関数終了時にreturn関連の情報をクリア
    cgs.return_value_ptr = None;
    cgs.return_label = None;
}

#[allow(dead_code)]
fn function_proto(function: FunctionProto, _cgs: &mut CodeGenStatus) {
    println!(
        "declare {} @{}({})",
        function
            .sig
            .symbol
            .get_type()
            .unwrap()
            .as_func()
            .unwrap()
            .return_type
            .to_llvm_format(),
        function.sig.symbol.ident.get_name(),
        function
            .sig
            .symbol
            .get_type()
            .unwrap()
            .as_func()
            .unwrap()
            .params
            .iter()
            .map(|x| format!("{}", x.to_llvm_format()))
            .collect::<Vec<_>>()
            .join(", "),
    );
}

fn gen_top_level(top_level: TopLevel, cgs: &mut CodeGenStatus) {
    match top_level {
        TopLevel::FunctionDef(function) => function_def(function, cgs),
        TopLevel::FunctionProto(function) => function_proto(function, cgs), // 関数プロトタイプは無視
        TopLevel::Stmt(stmt) => super::stmt::stmt(stmt, cgs),
    }
}

pub fn generate_program(program: Program, cgs: &mut CodeGenStatus) {
    for item in program.items {
        gen_top_level(item, cgs);
    }

    for o in cgs.outpus.iter() {
        match o {
            StackCommand::Push(this) => println!("Push {}", this.oneline()),
            StackCommand::BinaryOP(this) => println!("{:?}", this),
            StackCommand::Symbol(this) => println!("Symbol {}", this.oneline()),
            StackCommand::Alloca(this) => println!("Alloca {}", this.oneline()),
            StackCommand::Store => println!("{:?}", o), //　計算結果が下　対象は上
            StackCommand::Load => println!("{:?}", o),
            StackCommand::Pop => println!("{:?}", o),
            StackCommand::Label(this) => println!("Label {:?}", this), // ラベル定義
            StackCommand::Jump(this) => println!("Jump {:?}", this),
            StackCommand::JumpIfFalse(this) => println!("JumpIfFalse {:?}", this),
            StackCommand::Call => println!("{:?}", o),
        }
    }
}
