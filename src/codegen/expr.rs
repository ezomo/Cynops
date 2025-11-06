use super::*;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => {
            gen_expr(*binary.lhs, cgs);
            gen_expr(*binary.rhs, cgs);
            cgs.outputs.push(binary.op.into());
        }
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                gen_expr(*assign.rhs, cgs);
                gen_expr_left(*assign.lhs, cgs);

                cgs.outputs.push(StackCommand::Store);
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
                gen_expr(*arg.clone(), cgs);
            }
            gen_expr_left(*call.func.clone(), cgs);
            cgs.outputs.push(StackCommand::Call(call.func.r#type));
            cgs.outputs.push(StackCommand::Label(return_point));
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
                cgs.outputs.push(StackCommand::Load(typed_expr.r#type));
            }

            UnaryOp::Minus => {}
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Ternary(ternary) => {}
        SemaExpr::Subscript(subscript) => {
            gen_expr(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outputs
                .push(StackCommand::IndexAccess(subscript.subject.r#type.clone()));
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
        SemaExpr::Comma(comma) => {
            for exper in comma.assigns {
                gen_expr(exper, cgs);
            }
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
                .push(StackCommand::IndexAccess(subscript.subject.r#type.clone()));
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
