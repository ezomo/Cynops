use super::*;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

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
                cgs.outputs.push(StackCommand::Store);

                // 非効率ではあるが，面倒なのだ
                gen_expr(*assign.lhs, cgs);
            }
            _ => unreachable!(),
        },
        SemaExpr::NumInt(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::NumFloat(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::Char(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::String(_) => cgs.outputs.push(typed_expr.into()),
        SemaExpr::Symbol(symbol) => {
            let ty = symbol.get_type().unwrap();
            match ty {
                Type::Func(_) => cgs.outputs.push(StackCommand::Symbol(symbol.clone())),
                Type::Array(_) => {
                    cgs.outputs.push(StackCommand::Symbol(symbol.clone()));
                    cgs.outputs.push(StackCommand::AcsessUseLa);
                }
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
            UnaryOp::Bang => {}

            UnaryOp::Tilde => {}
            UnaryOp::Ampersand => {
                gen_expr_left(*unary.expr.clone(), cgs);

                if !matches!(unary.expr.r#type, Type::Func(_)) {
                    cgs.outputs.pop();

                    cgs.outputs.push(StackCommand::Address);
                }
            }

            UnaryOp::Asterisk => {
                gen_expr(*unary.expr.clone(), cgs);

                cgs.outputs.push(StackCommand::AcsessUseGa);

                //配列は読み込めないのよ　どうしようかな？
                if typed_expr.r#type.as_array().is_none() {
                    cgs.outputs.push(StackCommand::Load(typed_expr.r#type));
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
            gen_expr(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outputs
                .push(StackCommand::IndexAccess(typed_expr.r#type.clone()));

            if !typed_expr.r#type.is_address() {
                cgs.outputs
                    .push(StackCommand::Load(typed_expr.r#type.clone()));
            }
        }
        SemaExpr::MemberAccess(member_access) => match member_access.kind {
            MemberAccessOp::Dot => match &member_access.base.r#type {
                Type::Union(_) => {}
                Type::Struct(_) => {}
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
        SemaExpr::String(_) => cgs.outputs.push(typed_expr.into()),

        SemaExpr::Symbol(ident) => match ident.get_type().unwrap() {
            Type::Func(_) => cgs.outputs.push(StackCommand::Symbol(ident)),
            _ => {
                cgs.outputs.push(StackCommand::Symbol(ident));
                cgs.outputs.push(StackCommand::AcsessUseLa);
            }
        },
        SemaExpr::Unary(unary) => match unary.op {
            UnaryOp::Bang => {}

            UnaryOp::Tilde => {}
            UnaryOp::Ampersand => gen_expr(*unary.expr, cgs),

            UnaryOp::Asterisk => {
                gen_expr_left(*unary.expr.clone(), cgs);
                cgs.outputs
                    .push(StackCommand::Load(unary.expr.clone().r#type));
                if !matches!(typed_expr.r#type, Type::Func(_)) {
                    cgs.outputs.push(StackCommand::AcsessUseGa);
                }
            }

            UnaryOp::Minus => {}
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Ternary(ternary) => {}
        SemaExpr::Subscript(subscript) => {
            gen_expr(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outputs
                .push(StackCommand::IndexAccess(typed_expr.r#type.clone()));
        }
        SemaExpr::MemberAccess(member_access) => match member_access.kind {
            MemberAccessOp::Dot => match &member_access.base.r#type {
                Type::Union(_) => {}
                Type::Struct(_) => {}
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        SemaExpr::Cast(cast) => {}

        _ => unreachable!("{:?}", typed_expr.expr.oneline()),
    }
}
