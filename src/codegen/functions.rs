use crate::{
    sema::ast::Call,
    sema::ast::{Symbol, TypedExpr},
};

use super::*;

pub fn codegen_if_fn<C, T, E>(cond: C, then: T, else_opt: Option<E>, cgs: &mut CodeGenStatus)
where
    C: FnOnce(&mut CodeGenStatus),
    T: FnOnce(&mut CodeGenStatus),
    E: FnOnce(&mut CodeGenStatus),
{
    match else_opt {
        Some(else_closure) => {
            let label_then = cgs.name_gen.slabel();
            let label_else = cgs.name_gen.slabel();
            let label_end = cgs.name_gen.slabel();

            cond(cgs);
            cgs.outputs
                .push(StackCommand::Branch(label_then, label_else));
            cgs.outputs.push(label_then.into());
            then(cgs);
            cgs.outputs.push(StackCommand::Goto(label_end));
            cgs.outputs.push(label_else.into());
            else_closure(cgs);
            cgs.outputs.push(StackCommand::Goto(label_end));
            cgs.outputs.push(label_end.into());
        }
        None => {
            let label_then = cgs.name_gen.slabel();
            let label_end = cgs.name_gen.slabel();

            cond(cgs);
            cgs.outputs
                .push(StackCommand::Branch(label_then, label_end));
            cgs.outputs.push(label_then.into());
            then(cgs);
            cgs.outputs.push(StackCommand::Goto(label_end));
            cgs.outputs.push(label_end.into());
        }
    }
}

pub fn codegen_while_fn<C, B>(cond: C, body: B, cgs: &mut CodeGenStatus)
where
    C: FnOnce(&mut CodeGenStatus),
    B: FnOnce(&mut CodeGenStatus),
{
    let label_start = cgs.name_gen.slabel();
    let label_body = cgs.name_gen.slabel();
    let label_end = cgs.name_gen.slabel();

    // Labelをまたいで行動することはできない 使用
    {
        cgs.outputs.push(StackCommand::Goto(label_start));
    }
    // ループの先頭ラベル
    cgs.outputs.push(StackCommand::Label(label_start));

    // 条件式を評価
    cond(cgs);

    // 条件が真なら body へ、偽なら end へ
    cgs.outputs
        .push(StackCommand::Branch(label_body, label_end));

    // ループ本体
    cgs.outputs.push(StackCommand::Label(label_body));
    body(cgs);

    // 本体実行後、再び条件評価へ戻る
    cgs.outputs.push(StackCommand::Goto(label_start));

    // 終了ラベル
    cgs.outputs.push(StackCommand::Label(label_end));
}

//
pub fn codegen_call_fn(call: Call, cgs: &mut CodeGenStatus) {
    let return_point = cgs.name_gen.slabel();
    if !call.func.r#type.as_func().unwrap().return_type.is_void() {
        cgs.outputs.push(StackCommand::Alloc(
            call.func
                .r#type
                .as_func()
                .unwrap()
                .return_type
                .as_ref()
                .clone(),
        ));
    }

    cgs.outputs.push(StackCommand::ReturnPoint(return_point));
    cgs.outputs.push(StackCommand::GlobalAddress);

    for arg in call.args.into_iter() {
        gen_expr(arg.clone(), cgs);
    }
    gen_expr_left(*call.func.clone(), cgs);
    cgs.outputs.push(StackCommand::Call(call.func.r#type));
    cgs.outputs.push(StackCommand::Label(return_point));
}
