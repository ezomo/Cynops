use crate::ast::*;
use crate::token::{Keyword, Token};

pub fn program(tokens: &mut Vec<Token>) -> Program {
    let mut code = Program::new();
    while !tokens.is_empty() {
        if is_next_type(tokens) && is_next_fn(&tokens[1..]) {
            let sig = function_sig(tokens);
            if consume(Token::LBrace, tokens) {
                // function definition
                code.items.push(TopLevel::function_def(sig, *block(tokens)));
            } else {
                // function prototype
                code.items.push(TopLevel::function_proto(sig));
                consume(Token::Semicolon, tokens);
            }
        } else {
            code.items.push(TopLevel::stmt(*stmt(tokens)));
        }
    }
    code
}

fn function_sig(tokens: &mut Vec<Token>) -> FunctionSig {
    FunctionSig::new(consume_type(tokens), declarator(tokens))
}

fn stmt(tokens: &mut Vec<Token>) -> Box<Stmt> {
    if consume(Token::r#return(), tokens) {
        let expr_opt = if consume(Token::Semicolon, tokens) {
            None
        } else {
            let tmp = expr(tokens);
            consume(Token::Semicolon, tokens);
            Some(tmp)
        };
        Stmt::r#return(expr_opt)
    } else if consume(Token::r#if(), tokens) {
        Stmt::r#if(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(tokens);
                consume(Token::RParen, tokens);
                tmp
            },
            *stmt(tokens),
            {
                if consume(Token::r#else(), tokens) {
                    Some(*stmt(tokens))
                } else {
                    None
                }
            },
        )
    } else if consume(Token::r#while(), tokens) {
        Stmt::r#while(
            {
                consume(Token::LParen, tokens);
                let tmp = expr(tokens);
                consume(Token::RParen, tokens);
                tmp
            },
            *stmt(tokens),
        )
    } else if consume(Token::r#do(), tokens) {
        let body = *stmt(tokens);
        if !consume(Token::r#while(), tokens) {
            panic!("expected 'while' after 'do' statement");
        }
        consume(Token::LParen, tokens);
        let condition = expr(tokens);
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
                    let tmp = expr(tokens);
                    consume(Token::Semicolon, tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::Semicolon, tokens) {
                    Some(Expr::num(0))
                } else {
                    let tmp = expr(tokens);
                    consume(Token::Semicolon, tokens);
                    Some(tmp)
                }
            },
            {
                if consume(Token::RParen, tokens) {
                    None
                } else {
                    let tmp = expr(tokens);
                    consume(Token::RParen, tokens);
                    Some(tmp)
                }
            },
            *stmt(tokens),
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
        Stmt::block(*block(tokens))
    } else if is_next_decl_stmt(tokens) {
        Stmt::decl_stmt(decl_stmt(tokens))
    } else if consume(Token::r#switch(), tokens) {
        consume(Token::LParen, tokens);
        let cond = expr(tokens);
        consume(Token::RParen, tokens);

        consume(Token::LBrace, tokens);
        let mut cases = Vec::new();
        while !consume(Token::RBrace, tokens) {
            let switch_case = case_clause(tokens);
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
        Stmt::label(name, *stmt(tokens))
    } else {
        let tmp = expr(tokens);
        if !consume(Token::Semicolon, tokens) {
            panic!("error");
        }
        Stmt::expr(tmp)
    }
}

fn case_clause(tokens: &mut Vec<Token>) -> SwitchCase {
    let get_tokens = |tokens: &mut Vec<Token>| {
        if !consume(Token::Colon, tokens) {
            panic!("expected ':' after case expression");
        }

        let mut stmts = vec![];
        while !is_next_switch_stmt(tokens) {
            stmts.push(stmt(tokens));
        }

        stmts
    };

    if consume(Token::case(), tokens) {
        SwitchCase::case(expr(tokens), get_tokens(tokens))
    } else if consume(Token::default(), tokens) {
        SwitchCase::default(get_tokens(tokens))
    } else {
        panic!("expected 'case' or 'default' in switch statement");
    }
}

fn decl_stmt(tokens: &mut Vec<Token>) -> DeclStmt {
    if consume(Token::r#struct(), tokens) {
        DeclStmt::Struct(struct_def(tokens))
    } else {
        DeclStmt::typed(consume_type(tokens), {
            let mut init_declarators = vec![init_declarator(tokens)];
            while consume(Token::Comma, tokens) {
                init_declarators.push(init_declarator(tokens));
            }
            consume(Token::Semicolon, tokens);
            init_declarators
        })
    }
}

fn init_declarator(tokens: &mut Vec<Token>) -> InitDeclarator {
    InitDeclarator::new(declarator(tokens), {
        if consume(Token::Equal, tokens) {
            Some(initializer(tokens))
        } else {
            None
        }
    })
}

fn initializer(tokens: &mut Vec<Token>) -> Initializer {
    if consume(Token::LBrace, tokens) {
        let tmp = Initializer::list(initializer_list(tokens));
        consume(Token::RBrace, tokens);
        tmp
    } else {
        Initializer::expr(expr(tokens))
    }
}

fn initializer_list(tokens: &mut Vec<Token>) -> Vec<Initializer> {
    let mut initializers = vec![initializer(tokens)];
    while consume(Token::Comma, tokens) {
        initializers.push(initializer(tokens));
    }
    initializers
}

fn declarator(tokens: &mut Vec<Token>) -> Declarator {
    let mut poiner_level = 0;
    while consume(Token::Asterisk, tokens) {
        poiner_level += 1;
    }

    if poiner_level == 0 {
        Declarator::direct(direct_declarator(tokens))
    } else {
        Declarator::pointer(poiner_level, direct_declarator(tokens))
    }
}

fn direct_declarator(tokens: &mut Vec<Token>) -> DirectDeclarator {
    let mut base = if consume(Token::LParen, tokens) {
        let inner = declarator(tokens);
        consume(Token::RParen, tokens);

        DirectDeclarator::paren(inner)
    } else if is_next_ident(tokens) {
        DirectDeclarator::ident(consume_ident(tokens))
    } else {
        panic!("{:?}", tokens);
    };

    // ★ ここで左再帰をループに展開
    loop {
        if consume(Token::LBracket, tokens) {
            let size = if !consume(Token::RBracket, tokens) {
                Some(expr(tokens))
            } else {
                None
            };
            consume(Token::RBracket, tokens);
            base = DirectDeclarator::array(base, size)
        } else if consume(Token::LParen, tokens) {
            let params = if !consume(Token::RParen, tokens) {
                Some(param_list(tokens))
            } else {
                None
            };
            consume(Token::RParen, tokens);
            base = DirectDeclarator::func(base, params)
        } else {
            break;
        }
    }

    base
}

fn struct_def(tokens: &mut Vec<Token>) -> Struct {
    Struct::new(consume_ident(tokens), {
        consume(Token::LBrace, tokens);
        let mut ms = vec![struct_member(tokens)];
        while !consume(Token::RBrace, tokens) {
            ms.push(struct_member(tokens));
        }

        consume(Token::Semicolon, tokens);
        ms
    })
}

fn struct_member(tokens: &mut Vec<Token>) -> StructMember {
    StructMember::new(consume_type(tokens), {
        let mut decs = vec![declarator(tokens)];
        while consume(Token::Comma, tokens) {
            decs.push(declarator(tokens));
        }
        consume(Token::Semicolon, tokens);
        decs
    })
}

fn block(tokens: &mut Vec<Token>) -> Box<Block> {
    let mut code = vec![];

    while !consume(Token::RBrace, tokens) {
        code.push(stmt(tokens));
    }

    Block::new(code)
}

fn expr(tokens: &mut Vec<Token>) -> Expr {
    *assign(tokens)
}

fn assign(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = conditional(tokens);
    if consume(Token::Equal, tokens) {
        node = Expr::assign(AssignOp::equal(), node, assign(tokens));
    } else if consume(Token::PlusEqual, tokens) {
        node = Expr::assign(AssignOp::plus_equal(), node, assign(tokens));
    } else if consume(Token::MinusEqual, tokens) {
        node = Expr::assign(AssignOp::minus_equal(), node, assign(tokens));
    } else if consume(Token::AsteriskEqual, tokens) {
        node = Expr::assign(AssignOp::asterisk_equal(), node, assign(tokens));
    } else if consume(Token::SlashEqual, tokens) {
        node = Expr::assign(AssignOp::slash_equal(), node, assign(tokens));
    } else if consume(Token::PercentEqual, tokens) {
        node = Expr::assign(AssignOp::percent_equal(), node, assign(tokens));
    } else if consume(Token::CaretEqual, tokens) {
        node = Expr::assign(AssignOp::caret_equal(), node, assign(tokens));
    } else if consume(Token::PipeEqual, tokens) {
        node = Expr::assign(AssignOp::pipe_equal(), node, assign(tokens));
    } else if consume(Token::LessLessEqual, tokens) {
        node = Expr::assign(AssignOp::less_less_equal(), node, assign(tokens));
    } else if consume(Token::GreaterGreaterEqual, tokens) {
        node = Expr::assign(AssignOp::greater_greater_equal(), node, assign(tokens));
    } else if consume(Token::AmpersandEqual, tokens) {
        node = Expr::assign(AssignOp::ampersand_equal(), node, assign(tokens));
    }
    node
}

fn conditional(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = logical_or(tokens);
    if consume(Token::Question, tokens) {
        let then_branch = expr(tokens);
        consume(Token::Colon, tokens);
        let else_branch = expr(tokens);
        node = Expr::ternary(node, then_branch, else_branch);
    }
    node
}

fn logical_or(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = logical_and(tokens);
    loop {
        if consume(Token::PipePipe, tokens) {
            node = Expr::binary(BinaryOp::pipe_pipe(), node, logical_and(tokens));
        } else {
            return node;
        }
    }
}

fn logical_and(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = equality(tokens);
    loop {
        if consume(Token::AmpersandAmpersand, tokens) {
            node = Expr::binary(BinaryOp::ampersand_ampersand(), node, equality(tokens));
        } else {
            return node;
        }
    }
}

fn equality(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = relational(tokens);
    loop {
        if consume(Token::EqualEqual, tokens) {
            node = Expr::binary(BinaryOp::equal_equal(), node, relational(tokens));
        } else if consume(Token::NotEqual, tokens) {
            node = Expr::binary(BinaryOp::not_equal(), node, relational(tokens));
        } else {
            return node;
        }
    }
}

fn relational(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_or(tokens);
    loop {
        if consume(Token::Less, tokens) {
            node = Expr::binary(BinaryOp::less(), node, bitwise_or(tokens));
        } else if consume(Token::LessEqual, tokens) {
            node = Expr::binary(BinaryOp::less_equal(), node, bitwise_or(tokens));
        } else if consume(Token::Greater, tokens) {
            node = Expr::binary(BinaryOp::greater(), node, bitwise_or(tokens));
        } else if consume(Token::GreaterEqual, tokens) {
            node = Expr::binary(BinaryOp::greater_equal(), node, bitwise_or(tokens));
        } else {
            return node;
        }
    }
}

fn bitwise_or(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_xor(tokens);
    loop {
        if consume(Token::Pipe, tokens) {
            node = Expr::binary(BinaryOp::pipe(), node, bitwise_xor(tokens));
        } else {
            return node;
        }
    }
}

fn bitwise_xor(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = bitwise_and(tokens);
    loop {
        if consume(Token::Caret, tokens) {
            node = Expr::binary(BinaryOp::caret(), node, bitwise_and(tokens));
        } else {
            return node;
        }
    }
}

fn bitwise_and(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = shift(tokens);
    loop {
        if consume(Token::Ampersand, tokens) {
            node = Expr::binary(BinaryOp::ampersand(), node, shift(tokens));
        } else {
            return node;
        }
    }
}

fn shift(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = add(tokens);
    loop {
        if consume(Token::LessLess, tokens) {
            node = Expr::binary(BinaryOp::less_less(), node, add(tokens));
        } else if consume(Token::GreaterGreater, tokens) {
            node = Expr::binary(BinaryOp::greater_greater(), node, add(tokens));
        } else {
            return node;
        }
    }
}

fn add(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = mul(tokens);
    loop {
        if consume(Token::Plus, tokens) {
            node = Expr::binary(BinaryOp::plus(), node, mul(tokens));
        } else if consume(Token::Minus, tokens) {
            node = Expr::binary(BinaryOp::minus(), node, mul(tokens));
        } else {
            return node;
        }
    }
}

fn mul(tokens: &mut Vec<Token>) -> Box<Expr> {
    let mut node = unary(tokens);
    loop {
        if consume(Token::Asterisk, tokens) {
            node = Expr::binary(BinaryOp::asterisk(), node, unary(tokens));
        } else if consume(Token::Slash, tokens) {
            node = Expr::binary(BinaryOp::slash(), node, unary(tokens));
        } else if consume(Token::Percent, tokens) {
            node = Expr::binary(BinaryOp::percent(), node, unary(tokens));
        } else {
            return node;
        }
    }
}

fn unary(tokens: &mut Vec<Token>) -> Box<Expr> {
    if consume(Token::Plus, tokens) {
        unary(tokens)
    } else if consume(Token::Minus, tokens) {
        Expr::unary(UnaryOp::minus(), unary(tokens))
    } else if consume(Token::Bang, tokens) {
        Expr::unary(UnaryOp::bang(), unary(tokens))
    } else if consume(Token::Tilde, tokens) {
        Expr::unary(UnaryOp::tilde(), unary(tokens))
    } else if consume(Token::Ampersand, tokens) {
        Expr::unary(UnaryOp::ampersand(), unary(tokens))
    } else if consume(Token::Asterisk, tokens) {
        Expr::unary(UnaryOp::asterisk(), unary(tokens))
    } else if consume(Token::PlusPlus, tokens) {
        Expr::unary(UnaryOp::plus_plus(), unary(tokens))
    } else if consume(Token::MinusMinus, tokens) {
        Expr::unary(UnaryOp::minus_minus(), unary(tokens))
    } else {
        Box::new(postfix(tokens))
    }
}

fn postfix(tokens: &mut Vec<Token>) -> Expr {
    let node = primary(tokens);
    if consume(Token::PlusPlus, tokens) {
        Expr::postfix(PostfixOp::plus_plus(), node)
    } else if consume(Token::MinusMinus, tokens) {
        Expr::postfix(PostfixOp::minus_minus(), node)
    } else if consume(Token::LParen, tokens) {
        let tmp = Expr::call(node, arg_list(tokens));
        consume(Token::RParen, tokens);
        tmp
    } else if consume(Token::LBracket, tokens) {
        let tmp = Expr::subscript(node, expr(tokens));
        consume(Token::RBracket, tokens);
        tmp
    } else {
        node
    }
}

fn primary(tokens: &mut Vec<Token>) -> Expr {
    // 次のトークンが"("なら、"(" expr ")"のはず
    if consume(Token::LParen, tokens) {
        let node = expr(tokens);
        let _ = consume(Token::RParen, tokens);
        return node;
    }
    // そうでなければ数値か変数か関数のはず

    if is_next_atom(tokens) {
        consume_atom(tokens)
    } else {
        Expr::ident(consume_ident(tokens))
    }
}

fn arg_list(tokens: &mut Vec<Token>) -> Vec<Box<Expr>> {
    let mut args = Vec::new();
    if !tokens.is_empty() && *tokens.first().unwrap() != Token::RParen {
        args.push(Box::new(expr(tokens)));
        while consume(Token::Comma, tokens) {
            args.push(Box::new(expr(tokens)));
        }
    }
    args
}

fn param_list(tokens: &mut Vec<Token>) -> ParamList {
    if consume(Token::void(), tokens) {
        ParamList::Void
    } else if !is_next_type(tokens) {
        // これは恐らくmain()のような書き方をしている
        //だだしいのはmain(void)だけと一応通過させる後に禁止するかも
        ParamList::Void
    } else {
        let mut params = vec![param(tokens)];

        while consume(Token::Comma, tokens) {
            params.push(param(tokens));
        }
        ParamList::Params(params)
    }
}

fn param(tokens: &mut Vec<Token>) -> Param {
    Param::new(consume_type(tokens), {
        if is_next_declarator(tokens) {
            Some(declarator(tokens))
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

    return matches!(next, Token::Num(_) | Token::Char(_));
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

fn is_next_type(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();

    return matches!(
        next,
        Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Void)
    );
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

fn is_next_decl_stmt(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    is_next_type(tokens) || tokens.first().unwrap() == &Token::Keyword(Keyword::Struct)
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
    } else {
        panic!()
    }
}

fn consume_ident(tokens: &mut Vec<Token>) -> Ident {
    if let Some(Token::Ident(name)) = tokens.first() {
        let name = name.clone();
        tokens.remove(0);
        Ident::new(name)
    } else {
        panic!("Expected identifier, found {:?}", tokens);
    }
}

fn consume_type(tokens: &mut Vec<Token>) -> Type {
    if tokens.is_empty() {
        panic!("Expected type, but no tokens available");
    }

    if let Some(Token::Keyword(kw)) = tokens.first() {
        let ty = match kw {
            Keyword::Int => Type::Int,
            Keyword::Char => Type::Char,
            Keyword::Void => Type::Void,
            _ => panic!("Expected type, found {:?}", kw),
        };
        tokens.remove(0); // consume the keyword
        ty
    } else {
        panic!("Expected type, found {:?}", tokens.first());
    }
}
