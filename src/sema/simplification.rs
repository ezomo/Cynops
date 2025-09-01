use crate::ast::*;

pub fn program(program: &mut Program) {
    for top_level in program.items.iter_mut() {
        match top_level {
            TopLevel::FunctionDef(this) => block(&mut this.body),
            TopLevel::FunctionProto(this) => {}
            TopLevel::Stmt(this) => stmt(this),
        }
    }
}

fn block(block: &mut Block) {
    for this in block.statements.iter_mut() {
        stmt(this);
    }
}

fn stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::ExprStmt(this) => expr(this),
        Stmt::DeclStmt(this) => {}
        Stmt::Control(this) => {}
        Stmt::Return(this) => {}
        Stmt::Goto(this) => {}
        Stmt::Label(this) => {}
        Stmt::Block(this) => {}
        _ => {}
    }
}

fn expr(expr: &mut Expr) {
    match expr {
        Expr::Assign(this) => assign(this),
        Expr::Binary(this) => {}
        Expr::Call(this) => {}
        Expr::Char(this) => {}
        Expr::String(this) => {}
        Expr::Ident(this) => {}
        Expr::NumInt(this) => {}
        Expr::NumFloat(this) => {}
        Expr::Postfix(this) => {}
        Expr::Subscript(this) => {}
        Expr::MemberAccess(this) => {}
        Expr::Ternary(this) => {}
        Expr::Unary(this) => {}
        Expr::Sizeof(this) => {}
        Expr::Cast(this) => {}
        Expr::Comma(this) => {}
    }
}

fn assign(assign: &mut Assign) {
    if assign.op == AssignOp::Equal {
        return; // そのまま
    }

    let lhs = assign.lhs.clone();
    let rhs = assign.rhs.clone();

    assign.rhs = Box::new(Expr::Binary(Binary {
        op: match assign.op {
            AssignOp::PlusEqual => BinaryOp::plus(),
            AssignOp::MinusEqual => BinaryOp::minus(),
            AssignOp::AsteriskEqual => BinaryOp::asterisk(),
            AssignOp::SlashEqual => BinaryOp::slash(),
            AssignOp::PercentEqual => BinaryOp::percent(),
            AssignOp::CaretEqual => BinaryOp::caret(),
            AssignOp::PipeEqual => BinaryOp::pipe(),
            AssignOp::LessLessEqual => BinaryOp::less_less(),
            AssignOp::GreaterGreaterEqual => BinaryOp::greater_greater(),
            AssignOp::AmpersandEqual => BinaryOp::ampersand(),
            AssignOp::Equal => unreachable!(),
        },
        lhs,
        rhs,
    }));
    assign.op = AssignOp::equal();
}
