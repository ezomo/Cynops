use super::*;
use crate::ast::*;

fn gen_function(function: FunctionDef, cgs: &mut CodeGenStatus) {
    let name = function.sig.ident.clone();
    let params = function.param_names.clone();
    let args: Vec<String> = params.iter().map(|_| cgs.name_gen.next()).collect();

    println!(
        "define i64 @{}({}) {{",
        name.get_name(),
        args.iter()
            .map(|x| format!("i64 noundef %{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // return用の変数とラベルを設定
    let return_ptr = cgs.name_gen.next();
    let return_label = "return_label".to_string();
    println!("%{} = alloca i64", return_ptr);

    cgs.return_value_ptr = Some(return_ptr.clone());
    cgs.return_label = Some(return_label.clone());

    // 引数の処理
    for (i, param_name) in params.iter().enumerate() {
        let ptr = cgs.name_gen.next();
        println!("%{} = alloca i64", ptr);
        println!("store i64 %{}, ptr %{}", args[i], ptr);
        cgs.variables.insert(param_name.clone(), ptr);
    }

    // 関数本体の処理
    for stmt in function.body.into_vec() {
        super::stmt::stmt(*stmt, cgs);
    }

    // 常にreturn_labelにジャンプ（return文がない場合のため）
    println!("br label %{}", return_label);

    // return_labelとreturn処理
    println!("{}:", return_label);
    println!("%val = load i64, ptr %{}", return_ptr);
    println!("ret i64 %val");

    println!("}}");

    // 関数終了時にreturn関連の情報をクリア
    cgs.return_value_ptr = None;
    cgs.return_label = None;
    cgs.variables.clear();
}

fn gen_top_level(top_level: TopLevel, cgs: &mut CodeGenStatus) {
    match top_level {
        TopLevel::FunctionDef(function_def) => gen_function(function_def, cgs),
        TopLevel::FunctionProto(_) => todo!(), // 関数プロトタイプは無視
        TopLevel::Stmt(stmt) => super::stmt::stmt(stmt, cgs),
    }
}

pub fn generate_program(program: Program, cgs: &mut CodeGenStatus) {
    for item in program.items {
        gen_top_level(item, cgs);
    }
}
