use super::ast as new_ast;
use super::ast::*;
use crate::ast as old_ast;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub fn program(program: &old_ast::Program, session: &mut Session) -> new_ast::Program {
    let mut new_items = Vec::new();

    for item in &program.items {
        match convert_toplevel(item, session) {
            Some(converted_item) => new_items.push(converted_item),
            None => continue, // スキップ
        }
    }

    new_ast::Program { items: new_items }
}

fn convert_toplevel(
    toplevel: &old_ast::TopLevel,
    session: &mut Session,
) -> Option<new_ast::TopLevel> {
    match toplevel {
        old_ast::TopLevel::FunctionDef(func_def) => Some(new_ast::TopLevel::FunctionDef(
            convert_function_def(func_def, session),
        )),
        old_ast::TopLevel::FunctionProto(func_proto) => Some(new_ast::TopLevel::FunctionProto(
            convert_function_proto(func_proto, session),
        )),
        old_ast::TopLevel::Stmt(stmt) => Some(new_ast::TopLevel::Stmt(convert_stmt(stmt, session))),
    }
}

fn convert_function_def(
    func_def: &old_ast::FunctionDef,
    session: &mut Session,
) -> new_ast::FunctionDef {
    // 関数シグネチャを変換
    let sig = convert_function_sig(&func_def.sig, session);

    // 関数をsessionに登録
    session.register_function(sig.ident.clone(), sig.ty.clone());

    // 新しいスコープを作成して関数本体を処理
    session.push_scope();

    // パラメータを現在のスコープに登録
    if let Some(func_type) = sig.ty.as_func() {
        for (param_name, param_type) in func_def.param_names.iter().zip(&func_type.params) {
            session.register_symbols(param_name.as_same(), param_type.clone());
        }
    }

    let body = convert_block(&func_def.body, session);

    session.pop_scope();

    new_ast::FunctionDef {
        sig,
        param_names: func_def.param_names.iter().map(|p| p.as_same()).collect(),
        body: *body,
    }
}

fn convert_function_proto(
    func_proto: &old_ast::FunctionProto,
    session: &mut Session,
) -> new_ast::FunctionProto {
    let sig = convert_function_sig(&func_proto.sig, session);

    // プロトタイプもsessionに登録
    session.register_function(sig.ident.clone(), sig.ty.clone());

    new_ast::FunctionProto { sig }
}

fn convert_function_sig(sig: &old_ast::FunctionSig, session: &mut Session) -> new_ast::FunctionSig {
    new_ast::FunctionSig {
        ty: convert_type(&sig.ty, session),
        ident: sig.ident.as_same(),
    }
}

fn convert_type(ty: &old_ast::Type, session: &mut Session) -> new_ast::Type {
    match ty {
        old_ast::Type::Void => new_ast::Type::Void,
        old_ast::Type::Int => new_ast::Type::Int,
        old_ast::Type::Double => new_ast::Type::Double,
        old_ast::Type::Char => new_ast::Type::Char,
        old_ast::Type::DotDotDot => new_ast::Type::DotDotDot,
        old_ast::Type::Func(func) => new_ast::Type::Func(new_ast::Func {
            return_type: Box::new(convert_type(&func.return_type, session)),
            params: func
                .params
                .iter()
                .map(|p| convert_type(p, session))
                .collect(),
        }),
        old_ast::Type::Struct(s) => new_ast::Type::Struct(convert_struct(s, session)),
        old_ast::Type::Union(u) => new_ast::Type::Union(convert_union(u, session)),
        old_ast::Type::Enum(e) => new_ast::Type::Enum(convert_enum(e, session)),
        old_ast::Type::Pointer(inner) => {
            new_ast::Type::Pointer(Box::new(convert_type(inner, session)))
        }
        old_ast::Type::Array(arr) => {
            new_ast::Type::Array(new_ast::Array {
                array_of: Box::new(convert_type(&arr.array_of, session)),
                length: arr.length.as_ref().and_then(|_| Some(0)), // 後で解決する
            })
        }
        old_ast::Type::Typedef(ident) => new_ast::Type::Typedef(ident.as_same()),
    }
}

fn convert_struct(s: &old_ast::Struct, session: &mut Session) -> new_ast::Struct {
    let members = s
        .member
        .iter()
        .map(|m| convert_member_decl(m, session))
        .collect();

    let converted = new_ast::Struct {
        ident: s.ident.as_ref().map(|i| i.as_same()),
        member: members,
    };

    // 名前がある場合はsessionに登録
    if let Some(ref ident) = converted.ident {
        session.register_symbols(ident.clone(), new_ast::Type::Struct(converted.clone()));
    }

    converted
}

fn convert_union(u: &old_ast::Union, session: &mut Session) -> new_ast::Union {
    let members = u
        .member
        .iter()
        .map(|m| convert_member_decl(m, session))
        .collect();

    let converted = new_ast::Union {
        ident: u.ident.as_ref().map(|i| i.as_same()),
        member: members,
    };

    // 名前がある場合はsessionに登録
    if let Some(ref ident) = converted.ident {
        session.register_symbols(ident.clone(), new_ast::Type::Union(converted.clone()));
    }

    converted
}

fn convert_enum(e: &old_ast::Enum, session: &mut Session) -> new_ast::Enum {
    let variants = e
        .variants
        .iter()
        .map(|v| convert_enum_member(v, session))
        .collect();

    let converted = new_ast::Enum {
        ident: e.ident.as_ref().map(|i| i.as_same()),
        variants,
    };

    // 名前がある場合はsessionに登録
    if let Some(ref ident) = converted.ident {
        session.register_symbols(ident.clone(), new_ast::Type::Enum(converted.clone()));
    }

    converted
}

fn convert_member_decl(m: &old_ast::MemberDecl, session: &mut Session) -> new_ast::MemberDecl {
    new_ast::MemberDecl {
        ident: m.ident.as_same(),
        ty: convert_type(&m.ty, session),
    }
}

fn convert_enum_member(m: &old_ast::EnumMember, session: &mut Session) -> new_ast::EnumMember {
    new_ast::EnumMember {
        ident: m.ident.as_same(),
        value: m.value,
    }
}

fn convert_stmt(stmt: &old_ast::Stmt, session: &mut Session) -> new_ast::Stmt {
    match stmt {
        old_ast::Stmt::ExprStmt(expr) => new_ast::Stmt::ExprStmt(convert_expr(expr, session)),
        old_ast::Stmt::DeclStmt(decl) => new_ast::Stmt::DeclStmt(convert_decl_stmt(decl, session)),
        old_ast::Stmt::Control(control) => {
            new_ast::Stmt::Control(convert_control(control, session))
        }
        old_ast::Stmt::Return(ret) => new_ast::Stmt::Return(new_ast::Return {
            value: ret
                .value
                .as_ref()
                .map(|v| Box::new(convert_expr(v, session))),
        }),
        old_ast::Stmt::Goto(goto) => new_ast::Stmt::Goto(new_ast::Goto {
            label: goto.label.as_same(),
        }),
        old_ast::Stmt::Label(label) => new_ast::Stmt::Label(new_ast::Label {
            name: label.name.as_same(),
            stmt: Box::new(convert_stmt(&label.stmt, session)),
        }),
        old_ast::Stmt::Block(block) => new_ast::Stmt::Block(*convert_block(block, session)),
        old_ast::Stmt::Break => new_ast::Stmt::Break,
        old_ast::Stmt::Continue => new_ast::Stmt::Continue,
    }
}

fn convert_block(block: &old_ast::Block, session: &mut Session) -> Box<new_ast::Block> {
    session.push_scope();

    let statements = block
        .statements
        .iter()
        .map(|stmt| Box::new(convert_stmt(stmt, session)))
        .collect();

    session.pop_scope();

    Box::new(new_ast::Block { statements })
}

fn convert_control(control: &old_ast::Control, session: &mut Session) -> new_ast::Control {
    match control {
        old_ast::Control::If(if_stmt) => new_ast::Control::If(new_ast::If {
            cond: Box::new(convert_expr(&if_stmt.cond, session)),
            then_branch: Box::new(convert_stmt(&if_stmt.then_branch, session)),
            else_branch: if_stmt
                .else_branch
                .as_ref()
                .map(|e| Box::new(convert_stmt(e, session))),
        }),
        old_ast::Control::While(while_stmt) => new_ast::Control::While(new_ast::While {
            cond: Box::new(convert_expr(&while_stmt.cond, session)),
            body: Box::new(convert_stmt(&while_stmt.body, session)),
        }),
        old_ast::Control::DoWhile(do_while) => new_ast::Control::DoWhile(new_ast::DoWhile {
            body: Box::new(convert_stmt(&do_while.body, session)),
            cond: Box::new(convert_expr(&do_while.cond, session)),
        }),
        old_ast::Control::For(for_stmt) => new_ast::Control::For(new_ast::For {
            init: for_stmt
                .init
                .as_ref()
                .map(|i| Box::new(convert_expr(i, session))),
            cond: for_stmt
                .cond
                .as_ref()
                .map(|c| Box::new(convert_expr(c, session))),
            step: for_stmt
                .step
                .as_ref()
                .map(|s| Box::new(convert_expr(s, session))),
            body: Box::new(convert_stmt(&for_stmt.body, session)),
        }),
        old_ast::Control::Switch(switch) => new_ast::Control::Switch(new_ast::Switch {
            cond: Box::new(convert_expr(&switch.cond, session)),
            cases: switch
                .cases
                .iter()
                .map(|c| convert_switch_case(c, session))
                .collect(),
        }),
    }
}

fn convert_switch_case(case: &old_ast::SwitchCase, session: &mut Session) -> new_ast::SwitchCase {
    match case {
        old_ast::SwitchCase::Case(c) => new_ast::SwitchCase::Case(new_ast::Case {
            const_expr: convert_expr(&c.const_expr, session),
            stmts: c
                .stmts
                .iter()
                .map(|s| Box::new(convert_stmt(s, session)))
                .collect(),
        }),
        old_ast::SwitchCase::Default(d) => new_ast::SwitchCase::Default(new_ast::DefaultCase {
            stmts: d
                .stmts
                .iter()
                .map(|s| Box::new(convert_stmt(s, session)))
                .collect(),
        }),
    }
}

fn convert_decl_stmt(decl: &old_ast::DeclStmt, session: &mut Session) -> new_ast::DeclStmt {
    match decl {
        old_ast::DeclStmt::InitVec(inits) => {
            let converted_inits = inits
                .iter()
                .map(|init| convert_init(init, session))
                .collect();
            new_ast::DeclStmt::InitVec(converted_inits)
        }
        old_ast::DeclStmt::Struct(s) => new_ast::DeclStmt::Struct(convert_struct(s, session)),
        old_ast::DeclStmt::Union(u) => new_ast::DeclStmt::Union(convert_union(u, session)),
        old_ast::DeclStmt::Enum(e) => new_ast::DeclStmt::Enum(convert_enum(e, session)),
        old_ast::DeclStmt::Typedef(typedef) => {
            let converted = new_ast::Typedef {
                type_name: typedef.type_name.as_same(),
                actual_type: Box::new(convert_type(&typedef.actual_type, session)),
            };

            // typedefをsessionに登録
            session.register_symbols(converted.type_name.clone(), *converted.actual_type.clone());

            new_ast::DeclStmt::Typedef(converted)
        }
    }
}

fn convert_init(init: &old_ast::Init, session: &mut Session) -> new_ast::Init {
    let member_decl = convert_member_decl(&init.r, session);

    // 変数をsessionに登録
    session.register_symbols(member_decl.ident.clone(), member_decl.ty.clone());

    new_ast::Init {
        r: member_decl,
        l: init.l.as_ref().map(|data| convert_init_data(data, session)),
    }
}

fn convert_init_data(data: &old_ast::InitData, session: &mut Session) -> new_ast::InitData {
    match data {
        old_ast::InitData::Expr(expr) => new_ast::InitData::Expr(convert_expr(expr, session)),
        old_ast::InitData::Compound(compounds) => new_ast::InitData::Compound(
            compounds
                .iter()
                .map(|c| convert_init_data(c, session))
                .collect(),
        ),
    }
}

fn convert_expr(expr: &old_ast::Expr, session: &mut Session) -> new_ast::TypedExpr {
    let sema_expr = match expr {
        old_ast::Expr::Assign(assign) => new_ast::SemaExpr::Assign(new_ast::Assign {
            op: assign.op.as_same(),
            lhs: Box::new(convert_expr(&assign.lhs, session)),
            rhs: Box::new(convert_expr(&assign.rhs, session)),
        }),
        old_ast::Expr::Binary(binary) => new_ast::SemaExpr::Binary(new_ast::Binary {
            op: binary.op.as_same(),
            lhs: Box::new(convert_expr(&binary.lhs, session)),
            rhs: Box::new(convert_expr(&binary.rhs, session)),
        }),
        old_ast::Expr::Call(call) => new_ast::SemaExpr::Call(new_ast::Call {
            func: Box::new(convert_expr(&call.func, session)),
            args: call
                .args
                .iter()
                .map(|a| Box::new(convert_expr(a, session)))
                .collect(),
        }),
        old_ast::Expr::Char(c) => new_ast::SemaExpr::Char(*c),
        old_ast::Expr::String(s) => new_ast::SemaExpr::String(s.clone()),
        old_ast::Expr::Ident(ident) => {
            // Identを解決してSymbolに変換
            let symbol = new_ast::Symbol::new(
                ident.as_same(),
                Rc::downgrade(&session.get_scope(&ident.as_same())),
            );
            new_ast::SemaExpr::Ident(symbol)
        }
        old_ast::Expr::NumInt(n) => new_ast::SemaExpr::NumInt(*n),
        old_ast::Expr::NumFloat(f) => new_ast::SemaExpr::NumFloat(*f),
        old_ast::Expr::Postfix(_postfix) => {
            // Postfixは新しいASTでは別の形で処理する必要があります
            // とりあえずUnresolvedにしておく
            return new_ast::TypedExpr::new(
                new_ast::Type::Unresolved,
                new_ast::SemaExpr::NumInt(0),
            );
        }
        old_ast::Expr::Subscript(subscript) => new_ast::SemaExpr::Subscript(new_ast::Subscript {
            subject: Box::new(convert_expr(&subscript.name, session)),
            index: Box::new(convert_expr(&subscript.index, session)),
        }),
        old_ast::Expr::MemberAccess(member) => {
            new_ast::SemaExpr::MemberAccess(new_ast::MemberAccess {
                base: Box::new(convert_expr(&member.base, session)),
                member: member.member.as_same(),
                kind: member.kind.clone(),
            })
        }
        old_ast::Expr::Ternary(ternary) => new_ast::SemaExpr::Ternary(new_ast::Ternary {
            cond: Box::new(convert_expr(&ternary.cond, session)),
            then_branch: Box::new(convert_expr(&ternary.then_branch, session)),
            else_branch: Box::new(convert_expr(&ternary.else_branch, session)),
        }),
        old_ast::Expr::Unary(unary) => new_ast::SemaExpr::Unary(new_ast::Unary {
            op: unary.op.as_same(),
            expr: Box::new(convert_expr(&unary.expr, session)),
        }),
        old_ast::Expr::Sizeof(sizeof) => {
            let converted_sizeof = match sizeof {
                old_ast::Sizeof::Type(ty) => new_ast::Sizeof::Type(convert_type(ty, session)),
                old_ast::Sizeof::Expr(expr) => {
                    new_ast::Sizeof::TypedExpr(Box::new(convert_expr(expr, session)))
                }
            };
            new_ast::SemaExpr::Sizeof(converted_sizeof)
        }
        old_ast::Expr::Cast(cast) => new_ast::SemaExpr::Cast(new_ast::Cast {
            r#type: Box::new(convert_type(&cast.r#type, session)),
            expr: Box::new(convert_expr(&cast.expr, session)),
        }),
        old_ast::Expr::Comma(comma) => new_ast::SemaExpr::Comma(new_ast::Comma {
            assigns: comma
                .assigns
                .iter()
                .map(|e| convert_expr(e, session))
                .collect(),
        }),
    };

    // とりあえずすべての式の型をUnresolvedにする
    new_ast::TypedExpr::new(new_ast::Type::Unresolved, sema_expr)
}
