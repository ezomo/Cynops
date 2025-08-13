use std::collections::HashMap;

use crate::ast::{Enum, EnumMember};
use crate::token::{Keyword, Token};

use crate::ast::Typedef;
use crate::ast::*;
use crate::typelib;
#[derive(Debug)]
pub struct ParseSession {
    pub typedef_stack: Vec<HashMap<Ident, Type>>,
    pub struct_stack: Vec<HashMap<Ident, Type>>,
    pub union_stack: Vec<HashMap<Ident, Type>>,
    pub enum_stack: Vec<HashMap<Ident, Type>>,
    pub variable_stack: Vec<HashMap<Ident, Type>>,
    pub function_map: HashMap<Ident, Type>,
}

impl ParseSession {
    pub fn new() -> Self {
        Self {
            typedef_stack: Vec::new(),
            struct_stack: Vec::new(),
            union_stack: Vec::new(),
            enum_stack: Vec::new(),
            variable_stack: Vec::new(),
            function_map: HashMap::new(),
        }
    }

    // 新しいスコープを開始
    pub fn push_scope(&mut self) {
        self.typedef_stack.push(HashMap::new());
        self.struct_stack.push(HashMap::new());
        self.union_stack.push(HashMap::new());
        self.enum_stack.push(HashMap::new());
        self.variable_stack.push(HashMap::new());
    }

    // 現在のスコープを終了
    pub fn pop_scope(&mut self) {
        if self.typedef_stack.len() > 1 {
            self.typedef_stack.pop();
        }
        if self.struct_stack.len() > 1 {
            self.struct_stack.pop();
        }
        if self.union_stack.len() > 1 {
            self.union_stack.pop();
        }
        if self.enum_stack.len() > 1 {
            self.enum_stack.pop();
        }
        if self.variable_stack.len() > 1 {
            self.variable_stack.pop();
        }
    }

    pub fn is_base_type(&self, token: &Token) -> bool {
        match token {
            Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Void)
            | Token::Keyword(Keyword::Double) => true,
            Token::Ident(ident) => {
                let ident = Ident {
                    name: ident.clone(),
                };
                // typedef_stackから下向きに検索
                for scope in self.typedef_stack.iter().rev() {
                    if scope.contains_key(&ident) {
                        return true;
                    }
                }
                // struct_stackから下向きに検索
                for scope in self.struct_stack.iter().rev() {
                    if scope.contains_key(&ident) {
                        return true;
                    }
                }
                // union_stackから下向きに検索
                for scope in self.union_stack.iter().rev() {
                    if scope.contains_key(&ident) {
                        return true;
                    }
                }
                // enum_stackから下向きに検索
                for scope in self.enum_stack.iter().rev() {
                    if scope.contains_key(&ident) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    // Token を Type に変換
    pub fn cast(&self, token: &Token) -> Option<Type> {
        match token {
            Token::Keyword(Keyword::Int) => Some(Type::Int),
            Token::Keyword(Keyword::Void) => Some(Type::Void),
            Token::Keyword(Keyword::Char) => Some(Type::Char),
            Token::Keyword(Keyword::Double) => Some(Type::Double),
            Token::Ident(ident) => {
                let ident = Ident {
                    name: ident.clone(),
                };
                // typedef_stackから下向きに検索
                for scope in self.typedef_stack.iter().rev() {
                    if let Some(type_) = scope.get(&ident) {
                        return Some(type_.clone());
                    }
                }
                // struct_stackから下向きに検索
                for scope in self.struct_stack.iter().rev() {
                    if let Some(type_) = scope.get(&ident) {
                        return Some(type_.clone());
                    }
                }
                // union_stackから下向きに検索
                for scope in self.union_stack.iter().rev() {
                    if let Some(type_) = scope.get(&ident) {
                        return Some(type_.clone());
                    }
                }
                // enum_stackから下向きに検索
                for scope in self.enum_stack.iter().rev() {
                    if let Some(type_) = scope.get(&ident) {
                        return Some(type_.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    // typedef名を現在のスコープに登録
    fn register_typedef(&mut self, name: Ident, ty: Type) {
        if let Some(current_scope) = self.typedef_stack.last_mut() {
            current_scope.insert(name, ty);
        }
    }

    // struct名を現在のスコープに登録
    fn register_struct(&mut self, name: Ident, members: Type) {
        if let Some(current_scope) = self.struct_stack.last_mut() {
            current_scope.insert(name, members);
        }
    }

    // union名を現在のスコープに登録
    fn register_union(&mut self, name: Ident, members: Type) {
        if let Some(current_scope) = self.union_stack.last_mut() {
            current_scope.insert(name, members);
        }
    }

    // enum名を現在のスコープに登録
    fn register_enum(&mut self, name: Ident, variants: Type) {
        if let Some(current_scope) = self.enum_stack.last_mut() {
            current_scope.insert(name, variants);
        }
    }

    fn register_variable(&mut self, name: Ident, variants: Type) {
        if let Some(current_scope) = self.variable_stack.last_mut() {
            current_scope.insert(name, variants);
        }
    }

    fn get_variable(&self, name: &Ident) -> Option<Type> {
        for scope in self.variable_stack.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn get_function(&self, name: &Ident) -> Option<Type> {
        self.function_map.get(name).cloned()
    }

    fn get_var_fn(&self, name: &Ident) -> Option<Type> {
        if let Some(ty) = self.get_variable(name) {
            return Some(ty);
        }
        self.get_function(name)
    }

    fn register_function(&mut self, name: Ident, variants: Type) {
        self.function_map.insert(name, variants);
    }
}

pub fn program(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Program {
    let _is_next_composite_type_def = |tokens: &mut Vec<Token>| {
        is_next_composite_type_def(tokens, Token::r#enum())
            || is_next_composite_type_def(tokens, Token::r#struct())
            || is_next_composite_type_def(tokens, Token::r#union())
    };
    _parse_session.push_scope();
    let mut code = Program::new();
    while !tokens.is_empty() {
        if is_next_type(&_parse_session, tokens)
            && !_is_next_composite_type_def(tokens)
            && matches!(typelib::get_type(_parse_session, tokens), Type::Func(_))
        {
            let sig = function_sig(_parse_session, tokens);

            _parse_session.register_function(sig.0.ident.clone(), sig.0.ty.clone());

            if consume(Token::LBrace, tokens) {
                _parse_session.push_scope();

                match sig.0.ty.clone() {
                    Type::Func(a) => {
                        (0..sig.1.len()).for_each(|i| {
                            _parse_session.register_variable(sig.1[i].clone(), a.params[i].clone())
                        });
                    }
                    _ => panic!(),
                };
                // function definition
                code.items.push(TopLevel::function_def(
                    sig.0,
                    sig.1,
                    *block(_parse_session, tokens),
                ));

                _parse_session.pop_scope();
            } else {
                // function prototype
                code.items.push(TopLevel::function_proto(sig.0));
                consume(Token::Semicolon, tokens);
            }
        } else {
            code.items
                .push(TopLevel::stmt(*stmt(_parse_session, tokens)));
        }
    }
    _parse_session.pop_scope();
    code
}

fn function_sig(
    _parse_session: &mut ParseSession,
    tokens: &mut Vec<Token>,
) -> (FunctionSig, Vec<Ident>) {
    let (types, mut ident) = typelib::consume_and_extract_idents(_parse_session, tokens);
    (FunctionSig::new(types, ident.remove(0)), ident)
}

fn stmt(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Stmt> {
    if consume(Token::r#return(), tokens) {
        let expr_opt = if consume(Token::Semicolon, tokens) {
            None
        } else {
            let tmp = expr(_parse_session, tokens);
            consume(Token::Semicolon, tokens);
            Some(tmp)
        };
        Stmt::r#return(expr_opt)
    } else if consume(Token::r#if(), tokens) {
        Stmt::r#if(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(_parse_session, tokens);
                consume(Token::RParen, tokens);
                tmp
            },
            *stmt(_parse_session, tokens),
            {
                if consume(Token::r#else(), tokens) {
                    Some(*stmt(_parse_session, tokens))
                } else {
                    None
                }
            },
        )
    } else if consume(Token::r#while(), tokens) {
        Stmt::r#while(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(_parse_session, tokens);
                consume(Token::RParen, tokens);
                tmp
            },
            *stmt(_parse_session, tokens),
        )
    } else if consume(Token::r#do(), tokens) {
        let body = *stmt(_parse_session, tokens);
        if !consume(Token::r#while(), tokens) {
            panic!("expected 'while' after 'do' statement");
        }
        consume(Token::LParen, tokens);
        let condition = expr(_parse_session, tokens);
        consume(Token::RParen, tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after do-while statement");
        }
        Stmt::do_while(body, condition)
    } else if consume(Token::r#for(), tokens) {
        consume(Token::LParen, tokens);
        Stmt::r#for(
            {
                if consume(Token::Semicolon, tokens) {
                    None
                } else {
                    let tmp = expr(_parse_session, tokens);
                    consume(Token::Semicolon, tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::Semicolon, tokens) {
                    Some(Expr::num_int(0))
                } else {
                    let tmp = expr(_parse_session, tokens);
                    consume(Token::Semicolon, tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::RParen, tokens) {
                    None
                } else {
                    let tmp = expr(_parse_session, tokens);
                    consume(Token::RParen, tokens);
                    Some(tmp)
                }
            },
            *stmt(_parse_session, tokens),
        )
    } else if consume(Token::r#break(), tokens) {
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after break statement");
        }
        Stmt::r#break()
    } else if consume(Token::r#continue(), tokens) {
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after continue statement");
        }
        Stmt::r#continue()
    } else if consume(Token::LBrace, tokens) {
        Stmt::block(*block(_parse_session, tokens))
    } else if is_next_decl_stmt(_parse_session, tokens) {
        Stmt::decl_stmt(decl_stmt(_parse_session, tokens))
    } else if consume(Token::r#switch(), tokens) {
        consume(Token::LParen, tokens);
        let cond = expr(_parse_session, tokens);
        consume(Token::RParen, tokens);

        consume(Token::LBrace, tokens);
        let mut cases = Vec::new();
        while !consume(Token::RBrace, tokens) {
            let switch_case = case_clause(_parse_session, tokens);
            cases.push(switch_case);
        }
        Stmt::r#switch(cond, cases)
    } else if consume(Token::r#goto(), tokens) {
        let label = consume_ident(tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("expected semicolon after goto statement");
        }
        Stmt::goto(label)
    } else if is_next_label(tokens) {
        let name = consume_ident(tokens);
        if !consume(Token::Colon, tokens) {
            panic!("expected colon after label statement");
        }
        Stmt::label(name, *stmt(_parse_session, tokens))
    } else {
        let tmp = expr(_parse_session, tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("{:?}", tokens);
        }
        Stmt::expr(tmp)
    }
}

fn case_clause(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> SwitchCase {
    let get_stmts = |_parse_session: &mut ParseSession, tokens: &mut Vec<Token>| {
        if !consume(Token::Colon, tokens) {
            panic!("expected ':' after case expression");
        }

        let mut stmts = vec![];
        while !is_next_switch_stmt(tokens) {
            stmts.push(stmt(_parse_session, tokens));
        }

        stmts
    };

    if consume(Token::case(), tokens) {
        SwitchCase::case(
            expr(_parse_session, tokens),
            get_stmts(_parse_session, tokens),
        )
    } else if consume(Token::default(), tokens) {
        SwitchCase::default(get_stmts(_parse_session, tokens))
    } else {
        panic!("expected 'case' or 'default' in switch statement");
    }
}

fn decl_stmt(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> DeclStmt {
    if is_next_composite_type_def(tokens, Token::r#struct()) {
        consume(Token::r#struct(), tokens);
        DeclStmt::r#struct(struct_def(_parse_session, tokens))
    } else if is_next_composite_type_def(tokens, Token::r#union()) {
        consume(Token::r#union(), tokens);
        DeclStmt::union(union_def(_parse_session, tokens))
    } else if is_next_composite_type_def(tokens, Token::r#enum()) {
        consume(Token::r#enum(), tokens);
        DeclStmt::r#enum(enum_def(_parse_session, tokens))
    } else if consume(Token::typedef(), tokens) {
        let tmp = DeclStmt::typedef(typedef_stmt(_parse_session, tokens));
        consume(Token::Semicolon, tokens);
        tmp
    } else {
        let tmp = DeclStmt::init_vec(init_vec(_parse_session, tokens));
        consume(Token::Semicolon, tokens);
        tmp
    }
}

fn init_vec(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Vec<Init> {
    consume(Token::r#struct(), tokens);
    consume(Token::r#enum(), tokens);
    consume(Token::r#union(), tokens);

    let mut vec = vec![];
    let base = tokens.remove(0);

    vec.push(init(_parse_session, {
        tokens.insert(0, base.clone());
        tokens
    }));
    while consume(Token::Comma, tokens) {
        vec.push(init(_parse_session, {
            tokens.insert(0, base.clone());
            tokens
        }));
    }
    vec
}

fn init(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Init {
    Init::new(
        {
            let (types, ident) = typelib::consume_and_extract_idents(_parse_session, tokens);
            _parse_session.register_variable(ident[0].clone(), types.clone());
            MemberDecl::new(ident[0].clone(), types)
        },
        {
            if consume(Token::Equal, tokens) {
                Some(init_data(_parse_session, tokens))
            } else {
                None
            }
        },
    )
}

fn init_data(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> InitData {
    if consume(Token::LBrace, tokens) {
        let mut elements = vec![];
        while !consume(Token::RBrace, tokens) {
            elements.push(init_data(_parse_session, tokens));
            consume(Token::Comma, tokens);
        }
        InitData::Compound(elements)
    } else {
        InitData::Expr(assign(_parse_session, tokens).to_typed_expr())
    }
}

fn typedef_stmt(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Typedef {
    let ident;
    let ty = if is_next_composite_type_def(tokens, Token::r#struct()) {
        consume(Token::r#struct(), tokens);
        let st = struct_def(_parse_session, tokens);
        if st.ident.is_some() {
            _parse_session.register_struct(
                st.ident.as_ref().unwrap().clone(),
                Type::r#struct(st.clone()),
            );
        }
        ident = consume_ident(tokens);
        Type::r#struct(st)
    } else if is_next_composite_type_def(tokens, Token::r#union()) {
        consume(Token::r#union(), tokens);
        let un = union_def(_parse_session, tokens);
        if un.ident.is_some() {
            _parse_session
                .register_union(un.ident.as_ref().unwrap().clone(), Type::union(un.clone()));
        }
        ident = consume_ident(tokens);
        Type::union(un)
    } else if is_next_composite_type_def(tokens, Token::r#enum()) {
        consume(Token::r#enum(), tokens);
        let en = enum_def(_parse_session, tokens);
        if en.ident.is_some() {
            _parse_session
                .register_enum(en.ident.as_ref().unwrap().clone(), Type::r#enum(en.clone()));
        }
        ident = consume_ident(tokens);
        Type::r#enum(en)
    } else {
        let (t, i) = typelib::consume_and_extract_idents(_parse_session, tokens);
        ident = i[0].clone();
        t
    };
    _parse_session.register_typedef(ident.clone(), ty.clone());
    Typedef::new(ident, ty)
}

fn struct_def(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Struct {
    let idn = if is_next_ident(tokens) {
        Some(consume_ident(tokens))
    } else {
        None
    };

    let st = Struct::new(idn.clone(), {
        consume(Token::LBrace, tokens);

        let mut ms = vec![];
        while !consume(Token::RBrace, tokens) {
            ms.push(decl_member_vec(_parse_session, tokens));
            consume(Token::Semicolon, tokens);
        }

        consume(Token::Semicolon, tokens);
        ms.into_iter().flatten().collect()
    });

    if idn.is_some() {
        _parse_session.register_struct(idn.unwrap(), Type::r#struct(st.clone()));
    }
    st
}

fn union_def(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Union {
    let idn = if is_next_ident(tokens) {
        Some(consume_ident(tokens))
    } else {
        None
    };

    let un = Union::new(idn.clone(), {
        consume(Token::LBrace, tokens);

        let mut ms = vec![];
        while !consume(Token::RBrace, tokens) {
            ms.push(decl_member_vec(_parse_session, tokens));
            consume(Token::Semicolon, tokens);
        }

        consume(Token::Semicolon, tokens);
        ms.into_iter().flatten().collect()
    });

    if idn.is_some() {
        _parse_session.register_struct(idn.unwrap(), Type::r#union(un.clone()));
    }
    un
}

fn decl_member(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> MemberDecl {
    let (types, ident) = typelib::consume_and_extract_idents(_parse_session, tokens);
    _parse_session.register_variable(ident[0].clone(), types.clone());
    MemberDecl::new(ident[0].clone(), types)
}

fn decl_member_vec(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Vec<MemberDecl> {
    let mut vec = vec![];
    let base = tokens.remove(0);

    vec.push(decl_member(_parse_session, {
        tokens.insert(0, base.clone());
        tokens
    }));
    while consume(Token::Comma, tokens) {
        vec.push(decl_member(_parse_session, {
            tokens.insert(0, base.clone());
            tokens
        }));
    }
    vec
}

fn enum_def(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Enum {
    let idn = if is_next_ident(tokens) {
        Some(consume_ident(tokens))
    } else {
        None
    };

    let em = Enum::new(idn.clone(), {
        consume(Token::LBrace, tokens);
        let tmp = enum_member(tokens);
        consume(Token::RBrace, tokens);
        consume(Token::Semicolon, tokens);
        tmp
    });

    if idn.is_some() {
        _parse_session.register_struct(idn.unwrap(), Type::r#enum(em.clone()));
    }
    em
}

fn enum_member(tokens: &mut Vec<Token>) -> Vec<EnumMember> {
    let mut members = Vec::new();

    loop {
        let name = consume_ident(tokens);

        let value = if consume(Token::Equal, tokens) {
            if let Token::NumInt(n) = tokens.first().unwrap() {
                let n = *n;
                tokens.remove(0);
                Some(n)
            } else {
                panic!("Expected number after '=' in enum member");
            }
        } else {
            None
        };

        members.push(EnumMember::new(name, value));

        if !consume(Token::Comma, tokens) {
            break;
        }
    }

    members
}

fn block(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Block> {
    let mut code = vec![];
    _parse_session.push_scope();
    while !consume(Token::RBrace, tokens) {
        code.push(stmt(_parse_session, tokens));
    }
    _parse_session.pop_scope();
    Block::new(code)
}

pub fn expr(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Expr {
    comma(_parse_session, tokens)
}

fn comma(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Expr {
    let mut assigns = vec![*assign(_parse_session, tokens)];
    while consume(Token::Comma, tokens) {
        assigns.push(*assign(_parse_session, tokens));
    }
    if assigns.len() > 1 {
        Expr::comma(assigns)
    } else {
        assigns.first().unwrap().clone()
    }
}

fn assign(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = conditional(_parse_session, tokens);
    if consume(Token::Equal, tokens) {
        node = Expr::assign(AssignOp::equal(), node, assign(_parse_session, tokens));
    } else if consume(Token::PlusEqual, tokens) {
        node = Expr::assign(AssignOp::plus_equal(), node, assign(_parse_session, tokens));
    } else if consume(Token::MinusEqual, tokens) {
        node = Expr::assign(
            AssignOp::minus_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::AsteriskEqual, tokens) {
        node = Expr::assign(
            AssignOp::asterisk_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::SlashEqual, tokens) {
        node = Expr::assign(
            AssignOp::slash_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::PercentEqual, tokens) {
        node = Expr::assign(
            AssignOp::percent_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::CaretEqual, tokens) {
        node = Expr::assign(
            AssignOp::caret_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::PipeEqual, tokens) {
        node = Expr::assign(AssignOp::pipe_equal(), node, assign(_parse_session, tokens));
    } else if consume(Token::LessLessEqual, tokens) {
        node = Expr::assign(
            AssignOp::less_less_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::GreaterGreaterEqual, tokens) {
        node = Expr::assign(
            AssignOp::greater_greater_equal(),
            node,
            assign(_parse_session, tokens),
        );
    } else if consume(Token::AmpersandEqual, tokens) {
        node = Expr::assign(
            AssignOp::ampersand_equal(),
            node,
            assign(_parse_session, tokens),
        );
    }
    node
}

fn conditional(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = logical_or(_parse_session, tokens);
    if consume(Token::Question, tokens) {
        let then_branch = expr(_parse_session, tokens);
        consume(Token::Colon, tokens);
        let else_branch = expr(_parse_session, tokens);
        node = Expr::ternary(node, then_branch, else_branch);
    }
    node
}

fn logical_or(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = logical_and(_parse_session, tokens);
    loop {
        if consume(Token::PipePipe, tokens) {
            node = Expr::binary(
                BinaryOp::pipe_pipe(),
                node,
                logical_and(_parse_session, tokens),
            );
        } else {
            return node;
        }
    }
}

fn logical_and(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = equality(_parse_session, tokens);
    loop {
        if consume(Token::AmpersandAmpersand, tokens) {
            node = Expr::binary(
                BinaryOp::ampersand_ampersand(),
                node,
                equality(_parse_session, tokens),
            );
        } else {
            return node;
        }
    }
}

fn equality(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = relational(_parse_session, tokens);
    loop {
        if consume(Token::EqualEqual, tokens) {
            node = Expr::binary(
                BinaryOp::equal_equal(),
                node,
                relational(_parse_session, tokens),
            );
        } else if consume(Token::NotEqual, tokens) {
            node = Expr::binary(
                BinaryOp::not_equal(),
                node,
                relational(_parse_session, tokens),
            );
        } else {
            return node;
        }
    }
}

fn relational(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_or(_parse_session, tokens);
    loop {
        if consume(Token::Less, tokens) {
            node = Expr::binary(BinaryOp::less(), node, bitwise_or(_parse_session, tokens));
        } else if consume(Token::LessEqual, tokens) {
            node = Expr::binary(
                BinaryOp::less_equal(),
                node,
                bitwise_or(_parse_session, tokens),
            );
        } else if consume(Token::Greater, tokens) {
            node = Expr::binary(
                BinaryOp::greater(),
                node,
                bitwise_or(_parse_session, tokens),
            );
        } else if consume(Token::GreaterEqual, tokens) {
            node = Expr::binary(
                BinaryOp::greater_equal(),
                node,
                bitwise_or(_parse_session, tokens),
            );
        } else {
            return node;
        }
    }
}

fn bitwise_or(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_xor(_parse_session, tokens);
    loop {
        if consume(Token::Pipe, tokens) {
            node = Expr::binary(BinaryOp::pipe(), node, bitwise_xor(_parse_session, tokens));
        } else {
            return node;
        }
    }
}

fn bitwise_xor(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_and(_parse_session, tokens);
    loop {
        if consume(Token::Caret, tokens) {
            node = Expr::binary(BinaryOp::caret(), node, bitwise_and(_parse_session, tokens));
        } else {
            return node;
        }
    }
}

fn bitwise_and(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = shift(_parse_session, tokens);
    loop {
        if consume(Token::Ampersand, tokens) {
            node = Expr::binary(BinaryOp::ampersand(), node, shift(_parse_session, tokens));
        } else {
            return node;
        }
    }
}

fn shift(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = add(_parse_session, tokens);
    loop {
        if consume(Token::LessLess, tokens) {
            node = Expr::binary(BinaryOp::less_less(), node, add(_parse_session, tokens));
        } else if consume(Token::GreaterGreater, tokens) {
            node = Expr::binary(
                BinaryOp::greater_greater(),
                node,
                add(_parse_session, tokens),
            );
        } else {
            return node;
        }
    }
}

fn add(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = mul(_parse_session, tokens);
    loop {
        if consume(Token::Plus, tokens) {
            node = Expr::binary(BinaryOp::plus(), node, mul(_parse_session, tokens));
        } else if consume(Token::Minus, tokens) {
            node = Expr::binary(BinaryOp::minus(), node, mul(_parse_session, tokens));
        } else {
            return node;
        }
    }
}

fn mul(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = unary(_parse_session, tokens);
    loop {
        if consume(Token::Asterisk, tokens) {
            node = Expr::binary(BinaryOp::asterisk(), node, unary(_parse_session, tokens));
        } else if consume(Token::Slash, tokens) {
            node = Expr::binary(BinaryOp::slash(), node, unary(_parse_session, tokens));
        } else if consume(Token::Percent, tokens) {
            node = Expr::binary(BinaryOp::percent(), node, unary(_parse_session, tokens));
        } else {
            return node;
        }
    }
}

fn unary(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Box<Expr> {
    if consume(Token::Plus, tokens) {
        unary(_parse_session, tokens)
    } else if consume(Token::Minus, tokens) {
        Expr::unary(UnaryOp::minus(), unary(_parse_session, tokens))
    } else if consume(Token::Bang, tokens) {
        Expr::unary(UnaryOp::bang(), unary(_parse_session, tokens))
    } else if consume(Token::Tilde, tokens) {
        Expr::unary(UnaryOp::tilde(), unary(_parse_session, tokens))
    } else if consume(Token::Ampersand, tokens) {
        Expr::unary(UnaryOp::ampersand(), unary(_parse_session, tokens))
    } else if consume(Token::Asterisk, tokens) {
        Expr::unary(UnaryOp::asterisk(), unary(_parse_session, tokens))
    } else if consume(Token::PlusPlus, tokens) {
        Expr::unary(UnaryOp::plus_plus(), unary(_parse_session, tokens))
    } else if consume(Token::MinusMinus, tokens) {
        Expr::unary(UnaryOp::minus_minus(), unary(_parse_session, tokens))
    } else if consume(Token::sizeof(), tokens) {
        if consume(Token::LParen, tokens) && is_next_type(_parse_session, tokens) {
            let tmp = Expr::sizeof(Sizeof::r#type(consume_type(_parse_session, tokens)));
            consume(Token::RParen, tokens);
            tmp
        } else {
            let tmp = Expr::sizeof(Sizeof::expr(expr(_parse_session, tokens)));
            consume(Token::RParen, tokens);
            tmp
        }
    } else if is_next_cast(_parse_session, tokens) {
        Expr::cast(
            {
                consume(Token::LParen, tokens);
                let tmp = consume_type(_parse_session, tokens);
                consume(Token::RParen, tokens);
                tmp
            },
            *unary(_parse_session, tokens),
        )
    } else {
        Box::new(postfix(_parse_session, tokens))
    }
}

fn postfix(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Expr {
    let psd = postfix_chain(_parse_session, tokens);
    let mut base = psd.base;
    let suffixes = psd.suffixes;

    suffixes.iter().for_each(|suffixe| match suffixe {
        PostfixSuffix::ArrayAcsess(index) => base = Expr::subscript(base.clone(), index.clone()),
        PostfixSuffix::ArgList(args) => base = Expr::call(base.clone(), args.clone()),
        PostfixSuffix::PostfixOp(op) => base = Expr::postfix(op.clone().clone(), base.clone()),
        PostfixSuffix::MemberAccess(op, indet) => {
            base = Expr::member_access(base.clone(), indet.clone(), op.clone())
        }
    });

    base
}

fn postfix_chain(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> PostfixChain {
    let node = PostfixChain::new(primary(_parse_session, tokens), {
        let mut pos_vec = vec![];
        while is_next_postfix_suffix(tokens) {
            if consume(Token::PlusPlus, tokens) {
                pos_vec.push(PostfixSuffix::plus_plus());
            } else if consume(Token::MinusMinus, tokens) {
                pos_vec.push(PostfixSuffix::minus_minus());
            } else if consume(Token::MinusGreater, tokens) {
                pos_vec.push(PostfixSuffix::MemberAccess(
                    MemberAccessOp::minus_greater(),
                    consume_ident(tokens),
                ));
            } else if consume(Token::Dot, tokens) {
                pos_vec.push(PostfixSuffix::MemberAccess(
                    MemberAccessOp::dot(),
                    consume_ident(tokens),
                ));
            } else if consume(Token::LParen, tokens) {
                pos_vec.push(PostfixSuffix::ArgList(arg_list(_parse_session, tokens)));
                consume(Token::RParen, tokens);
            } else if consume(Token::LBracket, tokens) {
                pos_vec.push(PostfixSuffix::ArrayAcsess(expr(_parse_session, tokens)));
                consume(Token::RBracket, tokens);
            }
        }
        pos_vec
    });

    node
}

fn primary(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Expr {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Token::LParen, tokens) {
        let node = expr(_parse_session, tokens);
        let _ = consume(Token::RParen, tokens);
        return node;
    }
    // そうでなければ数値か変数か関数のはず
    else if is_next_atom(tokens) {
        consume_atom(tokens)
    } else {
        // 変数か関数のはず
        let ident = consume_ident(tokens);
        Expr::variable(ident.clone(), _parse_session.get_var_fn(&ident).unwrap())
    }
}

fn arg_list(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Vec<Box<Expr>> {
    let mut args = Vec::new();
    if !tokens.is_empty() && tokens.first().unwrap() != &Token::RParen {
        args.push(Box::new(*assign(_parse_session, tokens)));
        while consume(Token::Comma, tokens) {
            args.push(Box::new(*assign(_parse_session, tokens)));
        }
    }
    args
}

fn consume(op: Token, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

    if *tokens.first().unwrap() != op {
        return false;
    }

    tokens.remove(0);
    return true;
}

fn is_next_atom(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(
        next,
        Token::NumInt(_) | Token::Char(_) | Token::String(_) | Token::NumFloat(_)
    );
}

fn is_next_ident(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Ident(_));
}

fn is_next_type(_parse_session: &ParseSession, tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(
        next,
        Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Double)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Void)
    ) || next == &Token::r#struct()
        || next == &Token::r#union()
        || next == &Token::r#enum()
        || _parse_session.is_base_type(tokens.first().unwrap());
}

fn is_next_switch_stmt(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return next == &Token::case() || next == &Token::default() || next == &Token::RBrace;
}

fn is_next_label(tokens: &[Token]) -> bool {
    if tokens.len() < 2 {
        return false;
    }
    return is_next_ident(tokens) && matches!(tokens[1], Token::Colon);
}

fn is_next_decl_stmt(_parse_session: &ParseSession, tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    is_next_type(_parse_session, tokens)
        || tokens.first().unwrap() == &Token::r#struct()
        || tokens.first().unwrap() == &Token::r#union()
        || tokens.first().unwrap() == &Token::r#enum()
        || tokens.first().unwrap() == &Token::typedef()
        || _parse_session.is_base_type(tokens.first().unwrap())
}

fn is_next_postfix_suffix(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    match tokens[0] {
        Token::LBracket
        | Token::LParen
        | Token::PlusPlus
        | Token::MinusMinus
        | Token::Dot
        | Token::MinusGreater => true,
        _ => false,
    }
}

fn is_next_composite_type_def(tokens: &[Token], op: Token) -> bool {
    if tokens.len() < 3 {
        return false;
    }
    // e.g. struct Foo { ...
    tokens[0] == op && (tokens[1] == Token::LBrace || tokens[2] == Token::LBrace)
}

fn is_next_cast(_parse_session: &ParseSession, tokens: &[Token]) -> bool {
    if tokens.len() < 3 {
        return false;
    }

    // ( int ) x のような構文を探す
    if tokens[0] != Token::LParen {
        return false;
    }

    let mut new_tokens = tokens.to_vec();
    new_tokens.remove(0);
    is_next_type(_parse_session, &new_tokens)
}

fn consume_atom(tokens: &mut Vec<Token>) -> Expr {
    if tokens.is_empty() {
        panic!("Expected atom, but no tokens available");
    }

    if let Some(Token::NumInt(n)) = tokens.first() {
        let n = n.clone();
        tokens.remove(0);
        Expr::num_int(n)
    } else if let Some(Token::Char(c)) = tokens.first() {
        let c = c.clone();
        tokens.remove(0);
        Expr::char_lit(c)
    } else if let Some(Token::String(string)) = tokens.first() {
        let string = string.clone();
        tokens.remove(0);
        Expr::string(string)
    } else if let Some(Token::NumFloat(f)) = tokens.first() {
        let f = f.clone();
        tokens.remove(0);
        Expr::num_float(f)
    } else {
        panic!()
    }
}

fn consume_ident(tokens: &mut Vec<Token>) -> Ident {
    let ident = get_ident(tokens);
    tokens.remove(0);
    ident
}

fn consume_type(_parse_session: &mut ParseSession, tokens: &mut Vec<Token>) -> Type {
    typelib::consume_type(_parse_session, tokens)
}

fn get_ident(tokens: &[Token]) -> Ident {
    if let Some(Token::Ident(name)) = tokens.first() {
        let name = name.clone();
        Ident::new(name)
    } else {
        panic!("Expected identifier, found {:?}", tokens);
    }
}
