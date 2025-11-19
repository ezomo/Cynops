use std::rc::Rc;

use super::bf::*;
use super::stack::*;
use super::stmt as gen_stmt;
use super::{CodeGenStatus, StackCommand};
use crate::codegen::SFunc;
use crate::codegen::second::SeStackCommand;
use crate::op::{Arithmetic, BinaryOp, Comparison, Logical, UnaryOp};
use crate::sema::ast::*;
use crate::visualize::Visualize;

fn function_def(function: FunctionDef, cgs: &mut CodeGenStatus) {
    cgs.outputs.clear();

    let func_end = cgs.name_gen.slabel();
    cgs.func_end = Some(func_end);

    for s in function.body.into_vec() {
        gen_stmt::stmt(*s, cgs);
    }

    cgs.outputs.push(StackCommand::Label(func_end));
    cgs.outputs.push(StackCommand::FramePop);

    let func = SFunc::new(
        function.sig,
        function.param_names,
        cgs.outputs.clone(),
        cgs.name_gen.slabel(),
    );

    cgs.funcs.push(func);
    cgs.outputs.clear();
}

#[allow(dead_code)]
fn function_proto(function: FunctionProto, cgs: &mut CodeGenStatus) {
    if &function.sig.symbol.ident == &"putchar".into()
        && function.sig.symbol.get_type().unwrap()
            == Type::Func(Func {
                return_type: Type::Void.into(),
                params: vec![Type::Int],
            })
    {
        outchar(function, cgs);
    } else if function.sig.symbol.ident == "getchar".into()
        && function.sig.symbol.get_type().unwrap()
            == Type::Func(Func {
                return_type: Type::Int.into(),
                params: vec![Type::Void],
            })
    {
        getchar(function, cgs);
    }
}

fn outchar(function: FunctionProto, cgs: &mut CodeGenStatus) {
    cgs.outputs.clear();

    let ojcet: Ident = "object".into();
    let child = ScopeNode::add_child(&function.sig.symbol.scope.get_scope().unwrap());
    let sy = Symbol::new(ojcet.clone(), ScopePtr::new(Rc::downgrade(&child)));
    child.borrow_mut().register_symbols(ojcet, Type::Int);
    cgs.outputs.push(StackCommand::Symbol(sy.clone()));
    cgs.outputs.push(StackCommand::AcsessUseLa);
    cgs.outputs.push(StackCommand::Load(Type::Int));
    cgs.outputs.push(StackCommand::SellOut);
    cgs.outputs.push(StackCommand::FramePop);

    let func = SFunc::new(
        function.sig,
        vec![sy.clone()],
        cgs.outputs.clone(),
        cgs.name_gen.slabel(),
    );
    cgs.funcs.push(func);
    cgs.outputs.clear();
}

fn getchar(function: FunctionProto, cgs: &mut CodeGenStatus) {
    cgs.outputs.clear();

    let func_end = cgs.name_gen.slabel();
    cgs.func_end = Some(func_end);

    {
        cgs.outputs.push(StackCommand::Input);
        cgs.outputs.push(StackCommand::Return);

        //いらないはずなんだけな TODO
        {
            cgs.outputs.push(StackCommand::Goto(cgs.func_end.unwrap()));
            cgs.outputs.push(StackCommand::Label(cgs.name_gen.slabel())); //未到達空間回避
        }
    }

    cgs.outputs.push(StackCommand::Label(func_end));
    cgs.outputs.push(StackCommand::FramePop);

    let func = SFunc::new(
        function.sig,
        vec![],
        cgs.outputs.clone(),
        cgs.name_gen.slabel(),
    );

    cgs.funcs.push(func);
    cgs.outputs.clear();
}

fn gen_top_level(top_level: TopLevel, cgs: &mut CodeGenStatus) {
    match top_level {
        TopLevel::FunctionDef(function) => function_def(function, cgs),
        TopLevel::FunctionProto(function) => function_proto(function, cgs), // 関数プロトタイプは無視
        TopLevel::Stmt(stmt) => super::stmt::stmt(stmt, cgs),
    }
}

pub fn generate_program(program: Program) {
    let mut cgs = CodeGenStatus::new();
    let (fine_ex, _session) = fine_expr("src/codegen/expr.c");
    // sessionは必要，
    {
        for item in fine_ex.items {
            gen_top_level(item, &mut cgs);
        }

        cgs.funcs.iter().for_each(|x| {
            if x.sig
                .symbol
                .ident
                .get_name()
                .parse::<InsertFunction>()
                .is_ok()
            {
                cgs.insert_function.insert(
                    x.sig
                        .symbol
                        .ident
                        .get_name()
                        .parse::<InsertFunction>()
                        .unwrap(),
                    x.sig.symbol.clone(),
                );
            }
        });
    }

    for item in program.items {
        gen_top_level(item, &mut cgs);
    }

    // cgs.funcs.iter().for_each(|x| {
    //     eprintln!("{}", x.sig.symbol.ident.name);
    //     dbg!(&x.body);
    // });

    // eprintln!("===");

    let s = super::second::start(cgs.funcs);

    let stream = s
        .iter()
        .map(|x| convert(x.clone()))
        .collect::<Vec<StackInst>>();

    // dbg!(&stream);
    let transpilation = translate(&stream);

    println!("{}", show_bf(&transpilation));
}

fn fine_expr(filename: impl ToString) -> (Program, Session) {
    use crate::*;

    let mut input = fs::read_to_string(filename.to_string()).unwrap();

    input = String::from_iter(normalized(input.chars()));

    preprocessor::remove_comments(&mut input);
    preprocessor::unescape_char_literals(&mut input);

    let mut token = lexer::tokenize(&input);
    let mut session = parser::ParseSession::new();
    let mut program: ast::Program = parser::program(&mut session, &mut token);

    let mut simp_session = Session::new();
    sema::simplification::program(&mut program, &mut simp_session);

    let mut sema_session = sema::ast::Session::new();
    let new_program = sema::convert::program(&program, &mut sema_session);
    let type_check_result = sema::r#type::program(&new_program, &mut sema_session);

    (type_check_result.result, sema_session)
}

fn convert(b: SeStackCommand) -> StackInst {
    match b {
        SeStackCommand::Push(usize) => StackInst::Push(usize as u16),
        SeStackCommand::Branch(a, b) => StackInst::Branch(a as u16, b as u16), //True ,False
        SeStackCommand::BinaryOP(op) => match op {
            BinaryOp::Arithmetic(a) => match a {
                Arithmetic::Plus => StackInst::Add,
                Arithmetic::Asterisk => StackInst::Mul,
                Arithmetic::Minus => StackInst::Sub,
                Arithmetic::Slash => StackInst::Div,
                Arithmetic::Percent => StackInst::Mod,
                _ => unreachable!(),
            },
            BinaryOp::Comparison(a) => match a {
                Comparison::EqualEqual => StackInst::Eq,
                Comparison::Greater => StackInst::Gr,
                Comparison::GreaterEqual => StackInst::GrEq,
                Comparison::Less => StackInst::Lt,
                Comparison::LessEqual => StackInst::LtEq,
                Comparison::NotEqual => StackInst::Neq,
            },
            BinaryOp::Logical(a) => match a {
                Logical::AmpersandAmpersand => StackInst::And,
                Logical::PipePipe => StackInst::Or,
            },
        }, // 二項演算子
        SeStackCommand::UnaryOp(op) => match op {
            UnaryOp::Minus => StackInst::Negate,
            _ => unreachable!(),
        },
        SeStackCommand::Alloc(address) => StackInst::Alloc(address), //型のサイズだけメモリ確保
        SeStackCommand::DeAlloc(a) => StackInst::Dealloc(a),         //型のサイズだけメモリ確保
        SeStackCommand::WriteAddr => StackInst::StkStr,
        SeStackCommand::ReadAddr => StackInst::StkRead,
        SeStackCommand::Label(address) => StackInst::Label(address as u16), // ラベル定義
        SeStackCommand::Goto => StackInst::Goto,
        SeStackCommand::Exit => StackInst::Exit,
        SeStackCommand::Comment(this) => StackInst::Comment(this), // 無条件ジャンプ
        SeStackCommand::SellOut => StackInst::PutChar,
        SeStackCommand::Copy => StackInst::Copy,
        SeStackCommand::Input => StackInst::Input,
    }
}
