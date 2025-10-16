use super::ast as new_ast;
use super::ast::*;
use crate::ast as old_ast;

pub fn program(program: &old_ast::Program, session: &mut Session) -> new_ast::Program {
    let mut new_items = Vec::new();

    for item in &program.items {
        match convert_toplevel(item, session) {
            Some(converted_item) => new_items.push(converted_item),
            None => continue, // スキップ
        }
    }

    new_ast::Program::new(new_items)
}

fn convert_toplevel(
    toplevel: &old_ast::TopLevel,
    session: &mut Session,
) -> Option<new_ast::TopLevel> {
    match toplevel {
        old_ast::TopLevel::FunctionDef(func_def) => Some(convert_function_def(func_def, session)),
        old_ast::TopLevel::FunctionProto(func_proto) => {
            Some(convert_function_proto(func_proto, session))
        }
        old_ast::TopLevel::Stmt(stmt) => Some(new_ast::TopLevel::stmt(convert_stmt(stmt, session))),
    }
}

fn convert_function_def(
    func_def: &old_ast::FunctionDef,
    session: &mut Session,
) -> new_ast::TopLevel {
    // 関数シグニチャを変換
    let sig = convert_function_sig(&func_def.sig, session);

    // 関数をsessionに登録
    session.register_function(
        sig.symbol.ident.clone(),
        sig.symbol.get_type().unwrap().clone(),
    );

    // 新しいスコープを作成して関数本体を処理
    session.push_scope();

    let mut symbols = Vec::new();
    // パラメータを現在のスコープに登録
    if let Some(func_type) = sig.symbol.get_type().unwrap().as_func() {
        for (param_name, param_type) in func_def.param_names.iter().zip(&func_type.params) {
            session.register_symbols(param_name.as_same(), param_type.clone());
            symbols.push(Symbol::new(param_name.as_same(), session.current_scope()));
        }
    }

    let body = convert_block(&func_def.body, session);

    session.pop_scope();

    new_ast::TopLevel::function_def(sig, symbols, *body)
}

fn convert_function_proto(
    func_proto: &old_ast::FunctionProto,
    session: &mut Session,
) -> new_ast::TopLevel {
    let sig = convert_function_sig(&func_proto.sig, session);

    // プロトタイプもsessionに登録
    session.register_function(
        sig.symbol.ident.clone(),
        sig.symbol.get_type().unwrap().clone(),
    );

    new_ast::TopLevel::function_proto(sig)
}

fn convert_function_sig(sig: &old_ast::FunctionSig, session: &mut Session) -> new_ast::FunctionSig {
    let ty = convert_type(&sig.ty, session);
    session.register_function(sig.ident.as_same(), ty);
    new_ast::FunctionSig::new(Symbol::new(sig.ident.as_same(), session.current_scope()))
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
        old_ast::Type::Struct(s) => session
            .get_type(&s.ident.clone().as_ref().unwrap().as_same())
            .unwrap(),
        old_ast::Type::Union(u) => new_ast::Type::union(convert_union(u, session)),
        old_ast::Type::Enum(e) => new_ast::Type::r#enum(convert_enum(e, session)),
        old_ast::Type::Pointer(inner) => new_ast::Type::pointer(convert_type(inner, session)),
        old_ast::Type::Array(arr) => new_ast::Type::Array(Array::new(
            convert_type(&arr.array_of, session),
            if arr.length.is_none() {
                None
            } else {
                Some(new_ast::TypedExpr::new(
                    new_ast::Type::Int,
                    new_ast::SemaExpr::NumInt(
                        super::r#type::resolve_typed_expr(
                            &convert_expr(&arr.length.clone().unwrap(), session),
                            session,
                        )
                        .result // TypeCheckResult<T> から T を取得
                        .eval_const()
                        .unwrap_or(0) // 定数評価に失敗した場合は0
                        .try_into()
                        .unwrap_or(0), // 型変換に失敗した場合は0
                    ),
                ))
            },
        )),
        old_ast::Type::Typedef(ident) => session.get_type(&ident.as_same()).unwrap(),
    }
}

fn convert_struct(s: &old_ast::Struct, session: &mut Session) -> new_ast::Struct {
    // 内部で同じ方を参照している可能性があるのでダミー
    // struct a{
    //     struct a *hoge;
    // }

    let ident = s.ident.as_ref().map(|i| i.as_same());
    if let Some(ref ident) = ident {
        session.register_symbols(
            ident.clone(),
            new_ast::Type::Typedef(Symbol::new(ident.clone(), session.current_scope())),
        );
    }

    // 構造体内部要素の名前空間は別
    session.push_scope();
    let members = s
        .member
        .iter()
        .map(|m| convert_member_decl(m, session))
        .collect();
    session.pop_scope();

    // idとして使用する固有ident
    let ident_id = s
        .ident
        .as_ref()
        .unwrap()
        .as_same()
        .with_suffix(session.id().to_string());

    let converted = new_ast::Struct::new(
        s.ident.as_ref().map(|i: &old_ast::Ident| i.as_same()),
        Symbol::new(ident_id.clone(), session.current_scope()),
        members,
    );

    // 名前がある場合はsessionに登録
    if let Some(ref ident) = converted.ident {
        session.register_symbols(ident.clone(), new_ast::Type::r#struct(converted.clone()));
    }

    //sybmolを使用してアクセスするために登録
    session.register_symbols(ident_id.clone(), new_ast::Type::r#struct(converted.clone()));

    converted
}

fn convert_union(u: &old_ast::Union, session: &mut Session) -> new_ast::Union {
    let members = u
        .member
        .iter()
        .map(|m| convert_member_decl(m, session))
        .collect();

    let converted = new_ast::Union::new(
        u.ident.as_ref().map(|i| i.as_same()),
        Symbol::new(
            Ident::new(session.id().to_string()),
            session.current_scope(),
        ),
        members,
    );

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
        .map(|v| {
            new_ast::EnumMember::new(
                Symbol::new(v.ident.as_same(), session.current_scope()),
                v.value,
            )
        })
        .collect();

    let converted = new_ast::Enum::new(e.ident.as_ref().map(|i| i.as_same()), variants);
    let ty = new_ast::Type::Enum(converted.clone());

    // 名前がある場合はsessionに登録
    if let Some(ref ident) = converted.ident {
        session.register_symbols(ident.clone(), ty.clone());
    }

    converted
        .variants
        .iter()
        .for_each(|x| session.register_symbols(x.symbol.ident.clone(), ty.clone()));

    converted
}

fn convert_member_decl(m: &old_ast::MemberDecl, session: &mut Session) -> Symbol {
    let ty = convert_type(&m.ty, session);
    session.register_symbols(m.ident.as_same(), ty);
    Symbol::new(m.ident.as_same(), session.current_scope())
}

fn convert_stmt(stmt: &old_ast::Stmt, session: &mut Session) -> new_ast::Stmt {
    match stmt {
        old_ast::Stmt::ExprStmt(expr) => new_ast::Stmt::expr(convert_expr(expr, session)),
        old_ast::Stmt::DeclStmt(decl) => new_ast::Stmt::decl_stmt(convert_decl_stmt(decl, session)),
        old_ast::Stmt::Control(control) => {
            new_ast::Stmt::control(convert_control(control, session))
        }
        old_ast::Stmt::Return(ret) => {
            new_ast::Stmt::r#return(ret.value.as_ref().map(|v| convert_expr(v, session)))
        }
        old_ast::Stmt::Goto(goto) => new_ast::Stmt::goto(goto.label.as_same()),
        old_ast::Stmt::Label(label) => {
            new_ast::Stmt::label(label.name.as_same(), convert_stmt(&label.stmt, session))
        }
        old_ast::Stmt::Block(block) => new_ast::Stmt::block(*convert_block(block, session)),
        old_ast::Stmt::Break => new_ast::Stmt::r#break(),
        old_ast::Stmt::Continue => new_ast::Stmt::r#continue(),
    }
}

fn convert_block(block: &old_ast::Block, session: &mut Session) -> Box<new_ast::Block> {
    session.push_scope();

    let statements = block
        .statements
        .iter()
        .map(|stmt| Box::new(convert_stmt(stmt, session)))
        .collect();

    let tmp = Box::new(new_ast::Block::new(statements, session.current_scope()));
    session.pop_scope();
    tmp
}

fn convert_control(control: &old_ast::Control, session: &mut Session) -> new_ast::Control {
    match control {
        old_ast::Control::If(if_stmt) => new_ast::Control::r#if(
            convert_expr(&if_stmt.cond, session),
            convert_stmt(&if_stmt.then_branch, session),
            if_stmt
                .else_branch
                .as_ref()
                .map(|e| convert_stmt(e, session)),
        ),
        old_ast::Control::While(while_stmt) => new_ast::Control::r#while(
            convert_expr(&while_stmt.cond, session),
            convert_stmt(&while_stmt.body, session),
        ),
        old_ast::Control::DoWhile(do_while) => new_ast::Control::r#do_while(
            convert_stmt(&do_while.body, session),
            convert_expr(&do_while.cond, session),
        ),
        old_ast::Control::For(for_stmt) => new_ast::Control::r#for(
            for_stmt.init.as_ref().map(|i| convert_expr(i, session)),
            for_stmt.cond.as_ref().map(|c| convert_expr(c, session)),
            for_stmt.step.as_ref().map(|s| convert_expr(s, session)),
            convert_stmt(&for_stmt.body, session),
        ),
        old_ast::Control::Switch(switch) => new_ast::Control::r#switch(
            convert_expr(&switch.cond, session),
            switch
                .cases
                .iter()
                .map(|c| convert_switch_case(c, session))
                .collect(),
        ),
    }
}

fn convert_switch_case(case: &old_ast::SwitchCase, session: &mut Session) -> new_ast::SwitchCase {
    match case {
        old_ast::SwitchCase::Case(c) => new_ast::SwitchCase::case(
            convert_expr(&c.const_expr, session),
            c.stmts
                .iter()
                .map(|s| Box::new(convert_stmt(s, session)))
                .collect(),
        ),
        old_ast::SwitchCase::Default(d) => new_ast::SwitchCase::default(
            d.stmts
                .iter()
                .map(|s| Box::new(convert_stmt(s, session)))
                .collect(),
        ),
    }
}

fn convert_decl_stmt(decl: &old_ast::DeclStmt, session: &mut Session) -> new_ast::DeclStmt {
    match decl {
        old_ast::DeclStmt::InitVec(inits) => {
            let converted_inits = inits
                .iter()
                .map(|init| convert_init(init, session))
                .collect();
            new_ast::DeclStmt::init_vec(converted_inits)
        }
        old_ast::DeclStmt::Struct(s) => new_ast::DeclStmt::r#struct(convert_struct(s, session)),
        old_ast::DeclStmt::Union(u) => new_ast::DeclStmt::union(convert_union(u, session)),
        old_ast::DeclStmt::Enum(e) => new_ast::DeclStmt::r#enum(convert_enum(e, session)),
        old_ast::DeclStmt::Typedef(typedef) => {
            let converted =
                new_ast::Symbol::new(typedef.type_name.as_same(), session.current_scope());

            // typedefをsessionに登録
            let ty = convert_type(&typedef.actual_type, session);
            session.register_symbols(converted.ident.clone(), ty);

            new_ast::DeclStmt::Typedef(converted)
        }
    }
}

fn convert_init(init: &old_ast::Init, session: &mut Session) -> new_ast::Init {
    let member_decl = convert_member_decl(&init.r, session);
    let converted_init_data = init.l.as_ref().map(|data| convert_init_data(data, session));

    new_ast::Init::new(member_decl, converted_init_data)
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
        old_ast::Expr::Assign(assign) => new_ast::SemaExpr::assign(
            assign.op,
            convert_expr(&assign.lhs, session),
            convert_expr(&assign.rhs, session),
        ),
        old_ast::Expr::Binary(binary) => new_ast::SemaExpr::binary(
            binary.op,
            convert_expr(&binary.lhs, session),
            convert_expr(&binary.rhs, session),
        ),
        old_ast::Expr::Call(call) => new_ast::SemaExpr::call(
            convert_expr(&call.func, session),
            call.args.iter().map(|a| convert_expr(a, session)).collect(),
        ),
        old_ast::Expr::Char(c) => new_ast::SemaExpr::char_lit(*c),
        old_ast::Expr::String(s) => new_ast::SemaExpr::string(s.clone()),
        old_ast::Expr::Ident(ident) => {
            // Identを解決してSymbolに変換
            let symbol =
                new_ast::Symbol::new(ident.as_same(), session.get_ident_scope(&ident.as_same()));
            new_ast::SemaExpr::ident(symbol)
        }
        old_ast::Expr::NumInt(n) => new_ast::SemaExpr::num_int(*n),
        old_ast::Expr::NumFloat(f) => new_ast::SemaExpr::num_float(*f),
        old_ast::Expr::Postfix(_postfix) => {
            // Postfixは新しいASTでは別の形で処理する必要があります
            // とりあえずUnresolvedにしておく
            return new_ast::TypedExpr::new(
                new_ast::Type::Unresolved,
                new_ast::SemaExpr::num_int(0),
            );
        }
        old_ast::Expr::Subscript(subscript) => new_ast::SemaExpr::subscript(
            convert_expr(&subscript.name, session),
            convert_expr(&subscript.index, session),
        ),
        old_ast::Expr::MemberAccess(member) => new_ast::SemaExpr::member_access(
            convert_expr(&member.base, session),
            member.member.as_same(),
            member.kind.clone(),
        ),
        old_ast::Expr::Ternary(ternary) => new_ast::SemaExpr::ternary(
            convert_expr(&ternary.cond, session),
            convert_expr(&ternary.then_branch, session),
            convert_expr(&ternary.else_branch, session),
        ),
        old_ast::Expr::Unary(unary) => {
            new_ast::SemaExpr::unary(unary.op, convert_expr(&unary.expr, session))
        }
        old_ast::Expr::Sizeof(sizeof) => {
            let converted_sizeof = match sizeof {
                old_ast::Sizeof::Type(ty) => new_ast::Sizeof::r#type(convert_type(ty, session)),
                old_ast::Sizeof::Expr(expr) => new_ast::Sizeof::expr(convert_expr(expr, session)),
            };
            new_ast::SemaExpr::sizeof(converted_sizeof)
        }
        old_ast::Expr::Cast(cast) => new_ast::SemaExpr::cast(
            convert_type(&cast.r#type, session),
            Type::Unresolved,
            convert_expr(&cast.expr, session),
        ),
        old_ast::Expr::Comma(comma) => new_ast::SemaExpr::comma(
            comma
                .assigns
                .iter()
                .map(|e| convert_expr(e, session))
                .collect(),
        ),
    };

    // とりあえずすべての式の型をUnresolvedにする
    new_ast::TypedExpr::new(new_ast::Type::Unresolved, sema_expr)
}
