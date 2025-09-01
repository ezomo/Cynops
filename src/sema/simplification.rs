use crate::ast::*;

pub fn program(program: &mut Program) {
    for top_level in program.items.iter_mut() {
        match top_level {
            TopLevel::FunctionDef(this) => block(&mut this.body),
            TopLevel::FunctionProto(_) => {}
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
        Stmt::ExprStmt(expr) => {
            // Box<Expr> で所有権を取り出す場合
            let expr_value = std::mem::replace(expr, Expr::NumInt(0));
            let new_expr = _expr(expr_value);
            *expr = new_expr;
        }
        Stmt::DeclStmt(_) => {}
        Stmt::Control(_) => {}
        Stmt::Return(_) => {}
        Stmt::Goto(_) => {}
        Stmt::Label(_) => {}
        Stmt::Block(this) => block(this),
        _ => {}
    }
}

fn _expr(expr: Expr) -> Expr {
    match expr {
        Expr::Assign(this) => assign(this),
        Expr::Unary(this) => unary(this),
        Expr::Postfix(this) => postfix(this),
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

fn postfix(postfix: Postfix) -> Expr {
    Expr::comma(vec![
        *Expr::assign(
            AssignOp::Equal,
            postfix.expr.clone(),
            Expr::binary(
                if postfix.op == PostfixOp::plus_plus() {
                    BinaryOp::plus()
                } else {
                    BinaryOp::minus()
                },
                postfix.expr.clone(),
                Box::new(Expr::NumInt(1)),
            ),
        ),
        *Expr::binary(
            if postfix.op == PostfixOp::plus_plus() {
                BinaryOp::minus()
            } else {
                BinaryOp::plus()
            },
            postfix.expr,
            Box::new(Expr::NumInt(1)),
        ),
    ])
}

fn unary(unary: Unary) -> Expr {
    match unary.op {
        UnaryOp::Minus => {
            let target = *unary.expr;
            Expr::Binary(Binary {
                lhs: Box::new(Expr::NumInt(0)),
                op: BinaryOp::minus(),
                rhs: Box::new(target),
            })
        }
        UnaryOp::MinusMinus | UnaryOp::PlusPlus => *Expr::assign(
            AssignOp::equal(),
            unary.expr.clone(),
            Expr::binary(
                if unary.op == UnaryOp::plus_plus() {
                    BinaryOp::plus()
                } else {
                    BinaryOp::minus()
                },
                unary.expr,
                Box::new(Expr::NumInt(1)),
            ),
        ),
        _ => Expr::Unary(unary),
    }
}

fn assign(assign: Assign) -> Expr {
    if assign.op == AssignOp::Equal {
        return *Expr::assign(assign.op, assign.lhs, Box::new(_expr(*assign.rhs)));
    }

    let lhs = assign.lhs;
    let rhs_expr = *assign.rhs; // Box から所有権を直接取り出す
    let rhs_expr = _expr(rhs_expr); // 再帰的に書き換え

    let assign = Expr::assign(
        AssignOp::equal(),
        lhs.clone(),
        Expr::binary(
            match assign.op {
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
            Box::new(_expr(rhs_expr)),
        ),
    );

    *assign
}
