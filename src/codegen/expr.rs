use super::*;
use crate::codegen::r#type::Size;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

fn load(ty: &Type) -> Vec<StackCommand> {
    vec![StackCommand::AcsessUseGa, StackCommand::Load(ty.clone())]
}

fn store(ty: &Type) -> Vec<StackCommand> {
    vec![StackCommand::AcsessUseGa, StackCommand::Store(ty.clone())]
}

impl BinaryOp {
    fn insert_map(&self, ty: &Type) -> Option<InsertFunction> {
        match self {
            Self::Comparison(com) => match com {
                Comparison::Greater if ty == &Type::Int => InsertFunction::Greater.into(),
                Comparison::Less if ty == &Type::Int => InsertFunction::Less.into(),
                Comparison::GreaterEqual if ty == &Type::Int => InsertFunction::GreaterEqual.into(),
                Comparison::LessEqual if ty == &Type::Int => InsertFunction::LessEqual.into(),

                Comparison::EqualEqual if ty == &Type::Double => InsertFunction::DoubleEqual.into(),
                Comparison::Greater if ty == &Type::Double => InsertFunction::DoubleGreater.into(),
                Comparison::Less if ty == &Type::Double => InsertFunction::DoubleLess.into(),
                _ => None,
            },
            Self::Arithmetic(ari) => match ari {
                Arithmetic::Slash if ty == &Type::Int => InsertFunction::Slash.into(),
                Arithmetic::Percent if ty == &Type::Int => InsertFunction::Mod.into(),
                Arithmetic::Plus if ty == &Type::Double => InsertFunction::DoubleAdd.into(),
                Arithmetic::Minus if ty == &Type::Double => InsertFunction::DoubleSub.into(),
                Arithmetic::Asterisk if ty == &Type::Double => InsertFunction::DoubleMul.into(),
                _ => None,
            },
            BinaryOp::Logical(logical) => match logical {
                Logical::AmpersandAmpersand => InsertFunction::Land.into(),
                _ => None,
            },
        }
    }
}

fn try_codegen_binop(cgs: &mut CodeGenStatus, key: InsertFunction, binary: Binary) -> bool {
    if let Some(func) = cgs.insert_function.get(&key) {
        codegen_call_fn(
            Call::new(func.clone().into(), vec![*binary.lhs, *binary.rhs]),
            cgs,
        );
        true
    } else {
        false
    }
}

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => {
            let inset_fn = binary.op.insert_map(&binary.lhs.r#type);
            if let Some(key) = inset_fn {
                if try_codegen_binop(cgs, key.clone(), binary.clone()) {
                    return;
                }
            }
            gen_expr(*binary.lhs, cgs);
            gen_expr(*binary.rhs, cgs);
            cgs.outputs.push(binary.op.into());
        }
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
                if cgs
                    .insert_function
                    .get(&InsertFunction::DoubleMinus)
                    .is_some()
                    && matches!(unary.expr.r#type, Type::Double)
                {
                    codegen_call_fn(
                        Call::new(
                            cgs.insert_function
                                .get(&InsertFunction::DoubleMinus)
                                .unwrap()
                                .clone()
                                .into(),
                            vec![*unary.expr],
                        ),
                        cgs,
                    );
                    return;
                }
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
            Sizeof::Type(ty) => cgs.outputs.push(StackCommand::Push(ty.size().into())),
            Sizeof::TypedExpr(num) => cgs
                .outputs
                .push(StackCommand::Push(num.as_ref().r#type.size().into())),
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
        SemaExpr::Cast(_cast) => {
            // TODO
        }

        _ => unreachable!("{:?}", typed_expr.expr.oneline()),
    }
}
