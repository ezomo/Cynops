use super::*;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

fn load(ty: &Type) -> Vec<StackCommand> {
    vec![StackCommand::AcsessUseGa, StackCommand::Load(ty.clone())]
}

fn store(ty: &Type) -> Vec<StackCommand> {
    vec![StackCommand::AcsessUseGa, StackCommand::Store(ty.clone())]
}

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => match binary.op {
            BinaryOp::Comparison(Comparison::Greater)
                if cgs.insert_function.get(&InsertFunction::Greater).is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Greater)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Comparison(Comparison::Less)
                if cgs.insert_function.get(&InsertFunction::Less).is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Less)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Comparison(Comparison::GreaterEqual)
                if cgs
                    .insert_function
                    .get(&InsertFunction::GreaterEqual)
                    .is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::GreaterEqual)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Comparison(Comparison::LessEqual)
                if cgs
                    .insert_function
                    .get(&InsertFunction::LessEqual)
                    .is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::LessEqual)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Arithmetic(Arithmetic::Percent)
                if cgs.insert_function.get(&InsertFunction::Mod).is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Mod)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Arithmetic(Arithmetic::Slash)
                if cgs.insert_function.get(&InsertFunction::Slash).is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Slash)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            BinaryOp::Logical(Logical::AmpersandAmpersand)
                if cgs.insert_function.get(&InsertFunction::Land).is_some() =>
            {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Land)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*binary.lhs, *binary.rhs],
                    ),
                    cgs,
                );
            }

            _ => {
                gen_expr(*binary.lhs, cgs);
                gen_expr(*binary.rhs, cgs);
                cgs.outputs.push(binary.op.into());
            }
        },
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                gen_expr(*assign.rhs, cgs);
                gen_expr_left(*assign.lhs.clone(), cgs);

                cgs.outputs.extend(store(&typed_expr.r#type));

                // 非効率ではあるが，面倒なのだ
                gen_expr(*assign.lhs, cgs);
            }
            _ => unreachable!(),
        },
        SemaExpr::NumInt(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::NumFloat(this) => codegen_call_fn(
            Call::new(
                cgs.insert_function
                    .get(&InsertFunction::InitDouble)
                    .unwrap()
                    .clone()
                    .into(),
                vec![
                    (this.into_inner() as usize).into(),
                    frac_as_usize(this).into(),
                ],
            ),
            cgs,
        ),
        SemaExpr::Char(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::String(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::Symbol(symbol) => {
            let ty = symbol.get_type().unwrap();
            match ty {
                Type::Func(_) => cgs.outputs.push(StackCommand::Symbol(symbol.clone())),
                _ => {
                    cgs.outputs.push(StackCommand::Symbol(symbol.clone()));
                    cgs.outputs.push(StackCommand::AcsessUseLa);
                    cgs.outputs
                        .push(StackCommand::Load(symbol.get_type().unwrap()));
                }
            }
        }
        SemaExpr::Call(call) => {
            cgs.outputs.push(StackCommand::Comment(format!(
                "Call to function: {}",
                call.func.oneline()
            )));

            codegen_call_fn(call, cgs);
        }
        SemaExpr::Unary(unary) => match unary.op {
            UnaryOp::Bang if cgs.insert_function.get(&InsertFunction::Not).is_some() => {
                codegen_call_fn(
                    Call::new(
                        cgs.insert_function
                            .get(&InsertFunction::Not)
                            .unwrap()
                            .clone()
                            .into(),
                        vec![*unary.expr],
                    ),
                    cgs,
                );
            }
            UnaryOp::Tilde => {
                gen_expr(*unary.expr, cgs);
                cgs.outputs.push(StackCommand::UnaryOp(UnaryOp::bang()));
            }
            UnaryOp::Ampersand => {
                gen_expr_left(*unary.expr.clone(), cgs);
            }
            UnaryOp::Asterisk => {
                gen_expr_left(*unary.expr.clone(), cgs);
                cgs.outputs.extend(load(&unary.expr.clone().r#type));

                if !matches!(typed_expr.r#type, Type::Func(_)) {
                    cgs.outputs.extend(load(&typed_expr.r#type));
                }
            }
            UnaryOp::Minus => {
                gen_expr(*unary.expr, cgs);
                cgs.outputs.push(StackCommand::UnaryOp(UnaryOp::minus()));
            }
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Ternary(ternary) => {
            codegen_call_fn(
                Call::new(
                    cgs.insert_function
                        .get(&InsertFunction::Ternary)
                        .unwrap()
                        .clone()
                        .into(),
                    vec![*ternary.cond, *ternary.then_branch, *ternary.else_branch],
                ),
                cgs,
            );
        }
        SemaExpr::Subscript(subscript) => {
            gen_expr_left(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outputs
                .push(StackCommand::IndexAccess(typed_expr.r#type.clone()));

            if !matches!(typed_expr.r#type, Type::Func(_)) {
                cgs.outputs.extend(load(&typed_expr.r#type));
            }
        }
        SemaExpr::MemberAccess(member_access) => match member_access.kind {
            MemberAccessOp::Dot => match &member_access.base.r#type {
                Type::Union(_) => {}
                Type::Struct(st) => {
                    gen_expr_left(*member_access.base.clone(), cgs);
                    let pos = st
                        .member
                        .iter()
                        .map(|x| x.ident.clone())
                        .position(|x| x == member_access.member);

                    let types = st.member.iter().map(|x| x.get_type().unwrap()).collect();
                    cgs.outputs
                        .push(StackCommand::MemberAccess(types, pos.unwrap()));

                    cgs.outputs.extend(load(&typed_expr.r#type));
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        SemaExpr::Sizeof(sizeof) => match sizeof {
            Sizeof::Type(ty) => {}
            Sizeof::TypedExpr(num) => {}
        },
        SemaExpr::Cast(cast) => {
            // どうせ今はintしかないので
            gen_expr(*cast.expr, cgs);
        }
        SemaExpr::Comma(mut comma) => {
            for exper in comma.assigns.drain(..comma.assigns.len() - 1) {
                let ty = exper.r#type.clone();
                gen_expr(exper, cgs);
                cgs.outputs.push(StackCommand::Pop(ty));
            }
            gen_expr(comma.assigns.pop().unwrap(), cgs);
        }
    }
}

pub fn gen_expr_left(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) {
    match typed_expr.expr {
        SemaExpr::Symbol(ident) => match ident.get_type().unwrap() {
            Type::Func(_) => cgs.outputs.push(StackCommand::Symbol(ident)),
            _ => {
                cgs.outputs.push(StackCommand::Symbol(ident));
                cgs.outputs.push(StackCommand::La2GaAddress);
            }
        },
        SemaExpr::Unary(unary) => match unary.op {
            UnaryOp::Bang => {}

            UnaryOp::Tilde => {}
            UnaryOp::Ampersand => gen_expr_left(*unary.expr, cgs),

            UnaryOp::Asterisk => {
                gen_expr_left(*unary.expr.clone(), cgs);
                cgs.outputs.extend(load(&unary.expr.clone().r#type));
                // if !matches!(typed_expr.r#type, Type::Func(_)) {
                //     cgs.outputs.push(StackCommand::La2GaAddress);
                // }
            }

            UnaryOp::Minus => {}
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Subscript(subscript) => {
            gen_expr_left(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outputs
                .push(StackCommand::IndexAccess(typed_expr.r#type.clone()));
        }
        SemaExpr::MemberAccess(member_access) => match member_access.kind {
            MemberAccessOp::Dot => match &member_access.base.r#type {
                Type::Union(_) => {}
                Type::Struct(st) => {
                    gen_expr_left(*member_access.base.clone(), cgs);
                    let pos = st
                        .member
                        .iter()
                        .map(|x| x.ident.clone())
                        .position(|x| x == member_access.member);

                    let types = st.member.iter().map(|x| x.get_type().unwrap()).collect();
                    cgs.outputs
                        .push(StackCommand::MemberAccess(types, pos.unwrap()));
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        SemaExpr::Cast(cast) => {}

        _ => unreachable!("{:?}", typed_expr.expr.oneline()),
    }
}
