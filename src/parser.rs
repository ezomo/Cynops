use crate::ast::*;
use crate::token::{Keyword, Token};

use std::collections::HashMap;

#[derive(Clone)]
pub struct ParseSession {
    pub typedef_stack: Vec<HashMap<Ident, (TypedefType, Declarator)>>,
    pub tokens: Vec<Token>,
}

impl ParseSession {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            typedef_stack: Vec::new(),
            tokens,
        }
    }

    pub fn enter_block(&mut self) {
        self.typedef_stack.push(HashMap::new());
    }

    pub fn add_type(&mut self, declarator: Declarator, typedeftype: TypedefType) {
        let key = self.extract_ident_from_declarator(declarator.clone());
        self.typedef_stack
            .last_mut()
            .unwrap()
            .insert(key, (typedeftype, declarator));
    }

    pub fn exit_block(&mut self) {
        self.typedef_stack.pop();
    }

    pub fn is_type(&self, ident: &Ident) -> bool {
        for scope in self.typedef_stack.iter().rev() {
            if scope.contains_key(ident) {
                return true;
            }
        }
        false
    }

    fn extract_ident_from_declarator(&self, declarator: Declarator) -> Ident {
        match declarator {
            Declarator::Pointer(p) => self.extract_ident_from_direct_declarator(p.inner.unwrap()),
            Declarator::Direct(di) => self.extract_ident_from_direct_declarator(di),
        }
    }

    fn extract_ident_from_direct_declarator(&self, directdeclarator: DirectDeclarator) -> Ident {
        match directdeclarator {
            DirectDeclarator::Array(a) => self.extract_ident_from_direct_declarator(*a.base),
            DirectDeclarator::Func(f) => self.extract_ident_from_direct_declarator(*f.base),
            DirectDeclarator::Paren(p) => self.extract_ident_from_declarator(*p),
            DirectDeclarator::Ident(i) => i,
        }
    }
}

pub fn program(parse_session: &mut ParseSession) -> Program {
    parse_session.enter_block();
    let mut code = Program::new();
    while !parse_session.tokens.is_empty() {
        if is_next_type(&parse_session) && is_next_fn(&&mut parse_session.tokens[1..]) {
            let sig = function_sig(parse_session);
            if consume(Token::LBrace, &mut parse_session.tokens) {
                // function definition
                code.items
                    .push(TopLevel::function_def(sig, *block(parse_session)));
            } else {
                // function prototype
                code.items.push(TopLevel::function_proto(sig));
                consume(Token::Semicolon, &mut parse_session.tokens);
            }
        } else {
            code.items.push(TopLevel::stmt(*stmt(parse_session)));
        }
    }
    parse_session.exit_block();
    code
}

fn function_sig(parse_session: &mut ParseSession) -> FunctionSig {
    FunctionSig::new(
        consume_type(&mut parse_session.tokens),
        declarator(parse_session),
    )
}

fn stmt(parse_session: &mut ParseSession) -> Box<Stmt> {
    if consume(Token::r#return(), &mut parse_session.tokens) {
        let expr_opt = if consume(Token::Semicolon, &mut parse_session.tokens) {
            None
        } else {
            let tmp = expr(parse_session);
            consume(Token::Semicolon, &mut parse_session.tokens);
            Some(tmp)
        };
        Stmt::r#return(expr_opt)
    } else if consume(Token::r#if(), &mut parse_session.tokens) {
        Stmt::r#if(
            {
                consume(Token::LParen, &mut parse_session.tokens);
                let tmp = expr(parse_session);
                consume(Token::RParen, &mut parse_session.tokens);
                tmp
            },
            *stmt(parse_session),
            {
                if consume(Token::r#else(), &mut parse_session.tokens) {
                    Some(*stmt(parse_session))
                } else {
                    None
                }
            },
        )
    } else if consume(Token::r#while(), &mut parse_session.tokens) {
        Stmt::r#while(
            {
                consume(Token::LParen, &mut parse_session.tokens);
                let tmp = expr(parse_session);
                consume(Token::RParen, &mut parse_session.tokens);
                tmp
            },
            *stmt(parse_session),
        )
    } else if consume(Token::r#do(), &mut parse_session.tokens) {
        let body = *stmt(parse_session);
        if !consume(Token::r#while(), &mut parse_session.tokens) {
            panic!("expected 'while' after 'do' statement");
        }
        consume(Token::LParen, &mut parse_session.tokens);
        let condition = expr(parse_session);
        consume(Token::RParen, &mut parse_session.tokens);
        if !consume(Token::Semicolon, &mut parse_session.tokens) {
            panic!("expected semicolon after do-while statement");
        }
        Stmt::do_while(body, condition)
    } else if consume(Token::r#for(), &mut parse_session.tokens) {
        consume(Token::LParen, &mut parse_session.tokens);
        Stmt::r#for(
            {
                if consume(Token::Semicolon, &mut parse_session.tokens) {
                    None
                } else {
                    let tmp = expr(parse_session);
                    consume(Token::Semicolon, &mut parse_session.tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::Semicolon, &mut parse_session.tokens) {
                    Some(Expr::num(0))
                } else {
                    let tmp = expr(parse_session);
                    consume(Token::Semicolon, &mut parse_session.tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::RParen, &mut parse_session.tokens) {
                    None
                } else {
                    let tmp = expr(parse_session);
                    consume(Token::RParen, &mut parse_session.tokens);
                    Some(tmp)
                }
            },
            *stmt(parse_session),
        )
    } else if consume(Token::r#break(), &mut parse_session.tokens) {
        if !consume(Token::Semicolon, &mut parse_session.tokens) {
            panic!("expected semicolon after break statement");
        }
        Stmt::r#break()
    } else if consume(Token::r#continue(), &mut parse_session.tokens) {
        if !consume(Token::Semicolon, &mut parse_session.tokens) {
            panic!("expected semicolon after continue statement");
        }
        Stmt::r#continue()
    } else if consume(Token::LBrace, &mut parse_session.tokens) {
        Stmt::block(*block(parse_session))
    } else if is_next_decl_stmt(parse_session) {
        Stmt::decl_stmt(decl_stmt(parse_session))
    } else if consume(Token::r#switch(), &mut parse_session.tokens) {
        consume(Token::LParen, &mut parse_session.tokens);
        let cond = expr(parse_session);
        consume(Token::RParen, &mut parse_session.tokens);

        consume(Token::LBrace, &mut parse_session.tokens);
        let mut cases = Vec::new();
        while !consume(Token::RBrace, &mut parse_session.tokens) {
            let switch_case = case_clause(parse_session);
            cases.push(switch_case);
        }
        Stmt::r#switch(cond, cases)
    } else if consume(Token::r#goto(), &mut parse_session.tokens) {
        let label = consume_ident(&mut parse_session.tokens);
        if !consume(Token::Semicolon, &mut parse_session.tokens) {
            panic!("expected semicolon after goto statement");
        }
        Stmt::goto(label)
    } else if is_next_label(&mut parse_session.tokens) {
        let name = consume_ident(&mut parse_session.tokens);
        if !consume(Token::Colon, &mut parse_session.tokens) {
            panic!("expected colon after label statement");
        }
        Stmt::label(name, *stmt(parse_session))
    } else {
        let tmp = expr(parse_session);
        if !consume(Token::Semicolon, &mut parse_session.tokens) {
            panic!("{:?}", &mut parse_session.tokens);
        }
        Stmt::expr(tmp)
    }
}

fn case_clause(parse_session: &mut ParseSession) -> SwitchCase {
    let get_parse_session = |parse_session: &mut ParseSession| {
        if !consume(Token::Colon, &mut parse_session.tokens) {
            panic!("expected ':' after case expression");
        }

        let mut stmts = vec![];
        while !is_next_switch_stmt(&parse_session.tokens) {
            stmts.push(stmt(parse_session));
        }

        stmts
    };

    if consume(Token::case(), &mut parse_session.tokens) {
        SwitchCase::case(expr(parse_session), get_parse_session(parse_session))
    } else if consume(Token::default(), &mut parse_session.tokens) {
        SwitchCase::default(get_parse_session(parse_session))
    } else {
        panic!("expected 'case' or 'default' in switch statement");
    }
}

fn decl_stmt(parse_session: &mut ParseSession) -> DeclStmt {
    if is_next_composite_type_def(&mut parse_session.tokens, Token::r#struct()) {
        consume(Token::r#struct(), &mut parse_session.tokens);
        DeclStmt::struct_decl(struct_def(parse_session))
    } else if is_next_composite_type_def(&mut parse_session.tokens, Token::r#union()) {
        consume(Token::r#union(), &mut parse_session.tokens);
        DeclStmt::union_decl(union_def(parse_session))
    } else if is_next_composite_type_def(&mut parse_session.tokens, Token::r#enum()) {
        consume(Token::r#enum(), &mut parse_session.tokens);
        DeclStmt::enum_decl(enum_def(parse_session))
    } else if consume(Token::typedef(), &mut parse_session.tokens) {
        DeclStmt::typedef_decl(typedef_stmt(parse_session))
    } else {
        DeclStmt::typed(consume_type(&mut parse_session.tokens), {
            let mut init_declarators = vec![init_declarator(parse_session)];
            while consume(Token::Comma, &mut parse_session.tokens) {
                init_declarators.push(init_declarator(parse_session));
            }
            consume(Token::Semicolon, &mut parse_session.tokens);
            init_declarators
        })
    }
}

fn typedef_stmt(parse_session: &mut ParseSession) -> Typedef {
    let ty = typedef_type(parse_session);

    let pretreatment = |parse_session: &mut ParseSession, ds: &mut Vec<_>| {
        let d = declarator(parse_session);
        parse_session.add_type(d.clone(), ty.clone());
        ds.push(d);
    };

    let declarators = {
        let mut ds = vec![];
        pretreatment(parse_session, &mut ds);
        while consume(Token::Comma, &mut parse_session.tokens) {
            pretreatment(parse_session, &mut ds);
        }
        consume(Token::Semicolon, &mut parse_session.tokens);
        ds
    };
    Typedef::new(ty, declarators)
}

fn typedef_type(parse_session: &mut ParseSession) -> TypedefType {
    if is_next_composite_type_def(&mut parse_session.tokens, Token::r#struct()) {
        consume(Token::r#struct(), &mut parse_session.tokens);
        TypedefType::struct_decl(struct_def(parse_session))
    } else if is_next_composite_type_def(&mut parse_session.tokens, Token::r#union()) {
        consume(Token::r#union(), &mut parse_session.tokens);
        TypedefType::union_decl(union_def(parse_session))
    } else if is_next_composite_type_def(&mut parse_session.tokens, Token::r#enum()) {
        consume(Token::r#enum(), &mut parse_session.tokens);
        TypedefType::enum_decl(enum_def(parse_session))
    } else {
        TypedefType::r#type(consume_type(&mut parse_session.tokens))
    }
}

fn init_declarator(parse_session: &mut ParseSession) -> InitDeclarator {
    InitDeclarator::new(declarator(parse_session), {
        if consume(Token::Equal, &mut parse_session.tokens) {
            Some(initializer(parse_session))
        } else {
            None
        }
    })
}

fn initializer(parse_session: &mut ParseSession) -> Initializer {
    if consume(Token::LBrace, &mut parse_session.tokens) {
        let tmp = Initializer::list(initializer_list(parse_session));
        consume(Token::RBrace, &mut parse_session.tokens);
        tmp
    } else {
        Initializer::expr(expr(parse_session))
    }
}

fn initializer_list(parse_session: &mut ParseSession) -> Vec<Initializer> {
    let mut initializers = vec![initializer(parse_session)];
    while consume(Token::Comma, &mut parse_session.tokens) {
        initializers.push(initializer(parse_session));
    }
    initializers
}

fn declarator(parse_session: &mut ParseSession) -> Declarator {
    let mut poiner_level = 0;
    while consume(Token::Asterisk, &mut parse_session.tokens) {
        poiner_level += 1;
    }

    if poiner_level == 0 {
        Declarator::direct(direct_declarator(parse_session).unwrap())
    } else {
        Declarator::pointer(poiner_level, direct_declarator(parse_session))
    }
}

fn direct_declarator(parse_session: &mut ParseSession) -> Option<DirectDeclarator> {
    let mut base = if consume(Token::LParen, &mut parse_session.tokens) {
        let inner = declarator(parse_session);
        consume(Token::RParen, &mut parse_session.tokens);

        Some(DirectDeclarator::paren(inner))
    } else if is_next_ident(&mut parse_session.tokens) {
        Some(DirectDeclarator::ident(consume_ident(
            &mut parse_session.tokens,
        )))
    } else {
        None
    };

    //ここで左再帰をループに展開
    loop {
        if consume(Token::LBracket, &mut parse_session.tokens) {
            let size = if !consume(Token::RBracket, &mut parse_session.tokens) {
                Some(expr(parse_session))
            } else {
                None
            };
            consume(Token::RBracket, &mut parse_session.tokens);
            base = Some(DirectDeclarator::array(base.unwrap(), size))
        } else if consume(Token::LParen, &mut parse_session.tokens) {
            let params = if !consume(Token::RParen, &mut parse_session.tokens) {
                Some(param_list(parse_session))
            } else {
                None
            };
            consume(Token::RParen, &mut parse_session.tokens);
            base = Some(DirectDeclarator::func(base.unwrap(), params))
        } else {
            break;
        }
    }

    base
}

fn struct_def(parse_session: &mut ParseSession) -> Struct {
    Struct::new(
        {
            if is_next_ident(&parse_session.tokens) {
                Some(consume_ident(&mut parse_session.tokens))
            } else {
                None
            }
        },
        {
            consume(Token::LBrace, &mut parse_session.tokens);

            let mut ms = vec![];
            while !consume(Token::RBrace, &mut parse_session.tokens) {
                ms.push(decl_member(parse_session));
            }

            consume(Token::Semicolon, &mut parse_session.tokens);
            ms
        },
    )
}

fn union_def(parse_session: &mut ParseSession) -> Union {
    Union::new(
        {
            if is_next_ident(&parse_session.tokens) {
                Some(consume_ident(&mut parse_session.tokens))
            } else {
                None
            }
        },
        {
            consume(Token::LBrace, &mut parse_session.tokens);
            let mut ms = vec![];
            while !consume(Token::RBrace, &mut parse_session.tokens) {
                ms.push(decl_member(parse_session));
            }

            consume(Token::Semicolon, &mut parse_session.tokens);
            ms
        },
    )
}

fn decl_member(parse_session: &mut ParseSession) -> MemberDecl {
    MemberDecl::new(consume_type(&mut parse_session.tokens), {
        let mut decs = vec![declarator(parse_session)];
        while consume(Token::Comma, &mut parse_session.tokens) {
            decs.push(declarator(parse_session));
        }
        consume(Token::Semicolon, &mut parse_session.tokens);
        decs
    })
}

fn enum_def(parse_session: &mut ParseSession) -> Enum {
    Enum::new(
        {
            if is_next_ident(&parse_session.tokens) {
                Some(consume_ident(&mut parse_session.tokens))
            } else {
                None
            }
        },
        {
            consume(Token::LBrace, &mut parse_session.tokens);
            let tmp = enum_member(parse_session);
            consume(Token::RBrace, &mut parse_session.tokens);
            consume(Token::Semicolon, &mut parse_session.tokens);
            tmp
        },
    )
}

fn enum_member(parse_session: &mut ParseSession) -> Vec<EnumMember> {
    let mut members = Vec::new();

    loop {
        let name = consume_ident(&mut parse_session.tokens);

        let value = if consume(Token::Equal, &mut parse_session.tokens) {
            if let Token::Num(n) = &mut parse_session.tokens.first().unwrap() {
                let n = *n;
                let _ = &parse_session.tokens.remove(0);
                Some(n)
            } else {
                panic!("Expected number after '=' in enum member");
            }
        } else {
            None
        };

        members.push(EnumMember::new(name, value));

        if !consume(Token::Comma, &mut parse_session.tokens) {
            break;
        }
    }

    members
}

fn block(parse_session: &mut ParseSession) -> Box<Block> {
    parse_session.enter_block();
    let mut code = vec![];

    while !consume(Token::RBrace, &mut parse_session.tokens) {
        code.push(stmt(parse_session));
    }
    parse_session.exit_block();

    Block::new(code)
}

fn expr(parse_session: &mut ParseSession) -> Expr {
    comma(parse_session)
}
fn comma(parse_session: &mut ParseSession) -> Expr {
    let mut assigns = vec![*assign(parse_session)];
    while consume(Token::Comma, &mut parse_session.tokens) {
        assigns.push(*assign(parse_session));
    }
    Expr::comma(assigns)
}

fn assign(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = conditional(parse_session);
    if consume(Token::Equal, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::equal(), node, assign(parse_session));
    } else if consume(Token::PlusEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::plus_equal(), node, assign(parse_session));
    } else if consume(Token::MinusEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::minus_equal(), node, assign(parse_session));
    } else if consume(Token::AsteriskEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::asterisk_equal(), node, assign(parse_session));
    } else if consume(Token::SlashEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::slash_equal(), node, assign(parse_session));
    } else if consume(Token::PercentEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::percent_equal(), node, assign(parse_session));
    } else if consume(Token::CaretEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::caret_equal(), node, assign(parse_session));
    } else if consume(Token::PipeEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::pipe_equal(), node, assign(parse_session));
    } else if consume(Token::LessLessEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::less_less_equal(), node, assign(parse_session));
    } else if consume(Token::GreaterGreaterEqual, &mut parse_session.tokens) {
        node = Expr::assign(
            AssignOp::greater_greater_equal(),
            node,
            assign(parse_session),
        );
    } else if consume(Token::AmpersandEqual, &mut parse_session.tokens) {
        node = Expr::assign(AssignOp::ampersand_equal(), node, assign(parse_session));
    }
    node
}

fn conditional(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = logical_or(parse_session);
    if consume(Token::Question, &mut parse_session.tokens) {
        let then_branch = expr(parse_session);
        consume(Token::Colon, &mut parse_session.tokens);
        let else_branch = expr(parse_session);
        node = Expr::ternary(node, then_branch, else_branch);
    }
    node
}

fn logical_or(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = logical_and(parse_session);
    loop {
        if consume(Token::PipePipe, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::pipe_pipe(), node, logical_and(parse_session));
        } else {
            return node;
        }
    }
}

fn logical_and(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = equality(parse_session);
    loop {
        if consume(Token::AmpersandAmpersand, &mut parse_session.tokens) {
            node = Expr::binary(
                BinaryOp::ampersand_ampersand(),
                node,
                equality(parse_session),
            );
        } else {
            return node;
        }
    }
}

fn equality(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = relational(parse_session);
    loop {
        if consume(Token::EqualEqual, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::equal_equal(), node, relational(parse_session));
        } else if consume(Token::NotEqual, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::not_equal(), node, relational(parse_session));
        } else {
            return node;
        }
    }
}

fn relational(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = bitwise_or(parse_session);
    loop {
        if consume(Token::Less, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::less(), node, bitwise_or(parse_session));
        } else if consume(Token::LessEqual, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::less_equal(), node, bitwise_or(parse_session));
        } else if consume(Token::Greater, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::greater(), node, bitwise_or(parse_session));
        } else if consume(Token::GreaterEqual, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::greater_equal(), node, bitwise_or(parse_session));
        } else {
            return node;
        }
    }
}

fn bitwise_or(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = bitwise_xor(parse_session);
    loop {
        if consume(Token::Pipe, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::pipe(), node, bitwise_xor(parse_session));
        } else {
            return node;
        }
    }
}

fn bitwise_xor(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = bitwise_and(parse_session);
    loop {
        if consume(Token::Caret, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::caret(), node, bitwise_and(parse_session));
        } else {
            return node;
        }
    }
}

fn bitwise_and(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = shift(parse_session);
    loop {
        if consume(Token::Ampersand, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::ampersand(), node, shift(parse_session));
        } else {
            return node;
        }
    }
}

fn shift(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = add(parse_session);
    loop {
        if consume(Token::LessLess, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::less_less(), node, add(parse_session));
        } else if consume(Token::GreaterGreater, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::greater_greater(), node, add(parse_session));
        } else {
            return node;
        }
    }
}

fn add(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = mul(parse_session);
    loop {
        if consume(Token::Plus, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::plus(), node, mul(parse_session));
        } else if consume(Token::Minus, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::minus(), node, mul(parse_session));
        } else {
            return node;
        }
    }
}

fn mul(parse_session: &mut ParseSession) -> Box<Expr> {
    let mut node = unary(parse_session);
    loop {
        if consume(Token::Asterisk, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::asterisk(), node, unary(parse_session));
        } else if consume(Token::Slash, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::slash(), node, unary(parse_session));
        } else if consume(Token::Percent, &mut parse_session.tokens) {
            node = Expr::binary(BinaryOp::percent(), node, unary(parse_session));
        } else {
            return node;
        }
    }
}

fn unary(parse_session: &mut ParseSession) -> Box<Expr> {
    if consume(Token::Plus, &mut parse_session.tokens) {
        unary(parse_session)
    } else if consume(Token::Minus, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::minus(), unary(parse_session))
    } else if consume(Token::Bang, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::bang(), unary(parse_session))
    } else if consume(Token::Tilde, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::tilde(), unary(parse_session))
    } else if consume(Token::Ampersand, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::ampersand(), unary(parse_session))
    } else if consume(Token::Asterisk, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::asterisk(), unary(parse_session))
    } else if consume(Token::PlusPlus, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::plus_plus(), unary(parse_session))
    } else if consume(Token::MinusMinus, &mut parse_session.tokens) {
        Expr::unary(UnaryOp::minus_minus(), unary(parse_session))
    } else if consume(Token::sizeof(), &mut parse_session.tokens) {
        if consume(Token::LParen, &mut parse_session.tokens) && is_next_type(parse_session) {
            let tmp = Expr::sizeof(Sizeof::r#type(consume_type(&mut parse_session.tokens)));
            consume(Token::RParen, &mut parse_session.tokens);
            tmp
        } else {
            let tmp = Expr::sizeof(Sizeof::expr(expr(parse_session)));
            consume(Token::RParen, &mut parse_session.tokens);
            tmp
        }
    } else if is_next_cast(parse_session) {
        Expr::cast(
            {
                let tmp = consume_type(&mut parse_session.tokens);
                consume(Token::RParen, &mut parse_session.tokens);
                tmp
            },
            expr(parse_session),
        )
    } else {
        Box::new(postfix(parse_session))
    }
}

fn postfix(parse_session: &mut ParseSession) -> Expr {
    let psd = postfix_chain(parse_session);
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

fn postfix_chain(parse_session: &mut ParseSession) -> PostfixChain {
    let node = PostfixChain::new(primary(parse_session), {
        let mut pos_vec = vec![];
        while is_next_postfix_suffix(&mut parse_session.tokens) {
            if consume(Token::PlusPlus, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::plus_plus());
            } else if consume(Token::MinusMinus, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::minus_minus());
            } else if consume(Token::MinusGreater, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::MemberAccess(
                    MemberAccessOp::minus_greater(),
                    consume_ident(&mut parse_session.tokens),
                ));
            } else if consume(Token::Dot, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::MemberAccess(
                    MemberAccessOp::dot(),
                    consume_ident(&mut parse_session.tokens),
                ));
            } else if consume(Token::LParen, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::ArgList(arg_list(parse_session)));
                consume(Token::RParen, &mut parse_session.tokens);
            } else if consume(Token::LBracket, &mut parse_session.tokens) {
                pos_vec.push(PostfixSuffix::ArrayAcsess(expr(parse_session)));
                consume(Token::RBracket, &mut parse_session.tokens);
            }
        }
        pos_vec
    });

    node
}

fn primary(parse_session: &mut ParseSession) -> Expr {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Token::LParen, &mut parse_session.tokens) {
        let node = expr(parse_session);
        let _ = consume(Token::RParen, &mut parse_session.tokens);
        return node;
    }
    // そうでなければ数値か変数か関数のはず

    if is_next_atom(&mut parse_session.tokens) {
        consume_atom(&mut parse_session.tokens)
    } else {
        Expr::ident(consume_ident(&mut parse_session.tokens))
    }
}

fn arg_list(parse_session: &mut ParseSession) -> Vec<Box<Expr>> {
    let mut args = Vec::new();
    if !parse_session.tokens.is_empty() && parse_session.tokens.first().unwrap() != &Token::RParen {
        args.push(Box::new(expr(parse_session)));
        while consume(Token::Comma, &mut parse_session.tokens) {
            args.push(Box::new(expr(parse_session)));
        }
    }
    args
}

fn param_list(parse_session: &mut ParseSession) -> ParamList {
    if consume(Token::void(), &mut parse_session.tokens) {
        ParamList::Void
    } else if !is_next_type(parse_session) {
        // これは恐らくmain()のような書き方をしている
        //だだしいのはmain(void)だけと一応通過させる後に禁止するかも
        ParamList::Void
    } else {
        let mut params = vec![param(parse_session)];

        while consume(Token::Comma, &mut parse_session.tokens) {
            params.push(param(parse_session));
        }
        ParamList::Params(params)
    }
}

fn param(parse_session: &mut ParseSession) -> Param {
    Param::new(consume_type(&mut parse_session.tokens), {
        if is_next_declarator(&parse_session.tokens) {
            Some(declarator(parse_session))
        } else {
            None
        }
    })
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

    return matches!(next, Token::Num(_) | Token::Char(_) | Token::String(_));
}

fn is_next_ident(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(next, Token::Ident(_));
}

fn is_next_fn(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if !is_next_ident(tokens) {
        return false;
    }
    if tokens.len() < 2 {
        return false;
    }
    let second = tokens.get(1).unwrap();
    return matches!(second, Token::LParen);
}

fn is_next_type(parse_session: &ParseSession) -> bool {
    let tokens = &parse_session.tokens;
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(
        next,
        Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Void)
    ) || next == &Token::r#struct()
        || next == &Token::r#union()
        || next == &Token::r#enum()
        || {
            if is_next_ident(tokens) {
                parse_session.is_type(&get_ident(tokens))
            } else {
                false
            }
        };
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

fn is_next_declarator(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return next == &Token::Asterisk || next == &Token::LParen || matches!(next, Token::Ident(_));
}

fn is_next_decl_stmt(parse_session: &ParseSession) -> bool {
    let tokens = &parse_session.tokens;
    if tokens.is_empty() {
        return false;
    }

    is_next_type(parse_session)
        || tokens.first().unwrap() == &Token::r#struct()
        || tokens.first().unwrap() == &Token::r#union()
        || tokens.first().unwrap() == &Token::r#enum()
        || tokens.first().unwrap() == &Token::typedef()
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
    // e.g. struct Foo { ... }
    tokens[0] == op && (tokens[1] == Token::LBrace || tokens[2] == Token::LBrace)
}

fn is_next_cast(parse_session: &ParseSession) -> bool {
    let tokens = &parse_session.tokens;

    if tokens.len() < 3 {
        return false;
    }

    // ( int ) x のような構文を探す
    if tokens[0] != Token::LParen {
        return false;
    }

    let mut new = parse_session.clone();
    new.tokens.remove(0);
    is_next_type(&new)
}

fn consume_atom(tokens: &mut Vec<Token>) -> Expr {
    if tokens.is_empty() {
        panic!("Expected atom, but no tokens available");
    }

    if let Some(Token::Num(n)) = tokens.first() {
        let n = n.clone();
        tokens.remove(0);
        Expr::num(n)
    } else if let Some(Token::Char(c)) = tokens.first() {
        let c = c.clone();
        tokens.remove(0);
        Expr::char_lit(c)
    } else if let Some(Token::String(string)) = tokens.first() {
        let string = string.clone();
        tokens.remove(0);
        Expr::string(string)
    } else {
        panic!()
    }
}

fn consume_ident(tokens: &mut Vec<Token>) -> Ident {
    let ident = get_ident(tokens);
    tokens.remove(0);
    ident
}

fn consume_type(tokens: &mut Vec<Token>) -> Type {
    if tokens.is_empty() {
        panic!("Expected type, but no tokens available");
    }

    if consume(Token::int(), tokens) {
        return Type::Int;
    } else if consume(Token::char(), tokens) {
        return Type::Char;
    } else if consume(Token::void(), tokens) {
        return Type::Void;
    } else if consume(Token::r#struct(), tokens) {
        return Type::Struct(consume_ident(tokens));
    } else if consume(Token::r#union(), tokens) {
        return Type::Union(consume_ident(tokens));
    } else if consume(Token::r#enum(), tokens) {
        return Type::Enum(consume_ident(tokens));
    } else if is_next_ident(tokens) {
        return Type::Typedef(consume_ident(tokens));
    } else {
        panic!("Expected type, found {:?}", tokens.first());
    }
}

fn get_ident(tokens: &[Token]) -> Ident {
    if let Some(Token::Ident(name)) = tokens.first() {
        let name = name.clone();
        Ident::new(name)
    } else {
        panic!("Expected identifier, found {:?}", tokens);
    }
}
