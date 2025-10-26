use super::*;
use crate::op::*;
use crate::sema::ast::*;

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => {
            load(gen_expr, *binary.lhs, cgs);
            load(gen_expr, *binary.rhs, cgs);
            cgs.outpus.push(binary.op.into());
        }
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                gen_expr(*assign.rhs, cgs);
                gen_expr(*assign.lhs, cgs);

                cgs.outpus.push(StackCommand::Store);
            }
            _ => unreachable!(),
        },
        SemaExpr::NumInt(_) => cgs.outpus.push(typed_expr.into()),
        SemaExpr::NumFloat(_) => cgs.outpus.push(typed_expr.into()),
        SemaExpr::Char(_) => cgs.outpus.push(typed_expr.into()),
        SemaExpr::String(_) => cgs.outpus.push(typed_expr.into()),
        SemaExpr::Symbol(ident) => cgs.outpus.push(StackCommand::Symbol(ident)),
        SemaExpr::Call(call) => {
            let return_point = cgs.name_gen.slabel();
            if !call.func.r#type.as_func().unwrap().return_type.is_void() {
                cgs.outpus.push(StackCommand::Alloc(
                    call.func
                        .r#type
                        .as_func()
                        .unwrap()
                        .return_type
                        .as_ref()
                        .clone(),
                ));
            }

            cgs.outpus.push(StackCommand::ReturnPoint(return_point));
            for arg in call.args.into_iter().rev() {
                load(gen_expr, *arg, cgs);
            }
            gen_expr(*call.func.clone(), cgs);
            cgs.outpus.push(StackCommand::Call(call.func.r#type));
            cgs.outpus.push(StackCommand::Label(return_point));
        }
        SemaExpr::Unary(unary) => match unary.op {
            UnaryOp::Bang => {}

            UnaryOp::Tilde => {}
            UnaryOp::Ampersand => gen_expr(*unary.expr, cgs),

            UnaryOp::Asterisk => {
                gen_expr(*unary.expr.clone(), cgs);
                cgs.outpus.push(StackCommand::Load(unary.expr.r#type));
            }

            UnaryOp::Minus => {}
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Ternary(ternary) => {}
        SemaExpr::Subscript(subscript) => {
            gen_expr(*subscript.subject.clone(), cgs);
            gen_expr(*subscript.index.clone(), cgs);
            cgs.outpus
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
        SemaExpr::Cast(cast) => {}
        SemaExpr::Comma(comma) => {
            for exper in comma.assigns {
                gen_expr(exper, cgs);
            }
        }
    }
}
