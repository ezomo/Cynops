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
        SemaExpr::Symbol(ident) => match typed_expr.r#type {
            Type::Array(_) => {}
            Type::Pointer(_) => {}
            Type::Func(_) => {}
            other => cgs.outpus.push(StackCommand::Symbol(ident)),
        },
        SemaExpr::Call(call) => {}
        SemaExpr::Unary(unary) => match unary.op {
            UnaryOp::Bang => {}

            UnaryOp::Tilde => {}
            UnaryOp::Ampersand => {}
            UnaryOp::Asterisk => {}

            UnaryOp::Minus => {}
            _ => unreachable!("use simplification"),
        },
        SemaExpr::Ternary(ternary) => {}
        SemaExpr::Subscript(subscript) => {}
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
