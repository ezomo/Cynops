use super::*;
use crate::ast::*;

fn function_def(function: FunctionDef, cgs: &mut CodeGenStatus) {
    let args: Vec<(Ident, Type)> = (0..function.param_names.len())
        .map(|i| {
            (
                function.param_names[i].clone(),
                function.sig.ty.as_func().unwrap().params[i].clone(),
            )
        })
        .collect();

    println!(
        "define {} @{}({}) {{",
        function
            .sig
            .ty
            .as_func()
            .unwrap()
            .return_type
            .to_llvm_format(),
        function.sig.ident.get_name(),
        args.iter()
            .map(|x| format!("{} %{}", x.1.to_llvm_format(), x.0.get_name()))
            .collect::<Vec<_>>()
            .join(", "),
    );

    // return用の変数とラベルを設定
    let return_ptr = cgs.name_gen.register();
    let return_label = "return_label".to_string();
    let return_type = function
        .sig
        .ty
        .as_func()
        .unwrap()
        .return_type
        .to_llvm_format();
    println!(
        "{} = alloca {}",
        return_ptr.to_string(),
        return_type.to_string()
    );

    cgs.return_value_ptr = Some(return_ptr.clone().to_string());
    cgs.return_label = Some(return_label.clone());

    // 引数の処理
    {
        for (ident, ty) in &args {
            let ptr = cgs.name_gen.register();
            println!("{} = alloca {}", ptr.to_string(), ty.to_llvm_format());
            cgs.variables.insert(ident.clone(), ptr.to_string());
        }
        for (ident, ty) in &args {
            println!(
                "store {} %{}, ptr {}",
                ty.to_llvm_format(),
                ident.get_name(),
                cgs.variables[&ident]
            );
        }
    }

    // 関数本体の処理
    for stmt in function.body.into_vec() {
        super::stmt::stmt(*stmt, cgs);
    }

    // 常にreturn_labelにジャンプ（return文がない場合のため）
    println!("br label %{}", return_label);

    // return_labelとreturn処理
    println!("{}:", return_label);
    println!(
        "%val = load {}, ptr {}",
        return_type,
        return_ptr.to_string()
    );
    println!("ret {} %val", return_type);

    println!("}}");

    // 関数終了時にreturn関連の情報をクリア
    cgs.return_value_ptr = None;
    cgs.return_label = None;
    cgs.variables.clear();
}

#[allow(dead_code)]
fn function_proto(function: FunctionProto, _cgs: &mut CodeGenStatus) {
    println!(
        "declare {} @{}({})",
        function
            .sig
            .ty
            .as_func()
            .unwrap()
            .return_type
            .to_llvm_format(),
        function.sig.ident.get_name(),
        function
            .sig
            .ty
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
        TopLevel::FunctionProto(_function) => return, // 関数プロトタイプは無視
        TopLevel::Stmt(stmt) => super::stmt::stmt(stmt, cgs),
    }
}

pub fn generate_program(program: Program, cgs: &mut CodeGenStatus) {
    for item in program.items {
        gen_top_level(item, cgs);
    }
}
