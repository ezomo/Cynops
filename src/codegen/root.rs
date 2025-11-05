use std::rc::Rc;

use super::stmt as gen_stmt;
use super::{CodeGenStatus, StackCommand};
use crate::codegen::SFunc;
use crate::codegen::second::SeStackCommand;
use crate::op::{Arithmetic, BinaryOp, Comparison};
use crate::{sema::ast::*, visualize::OneLine};

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
    if function.sig.symbol.ident == "cellout".into()
        && function.sig.symbol.get_type().unwrap()
            == Type::Func(Func {
                return_type: Type::Void.into(),
                params: vec![Type::Int],
            })
    {
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
    for item in program.items {
        gen_top_level(item, &mut cgs);
    }

    first_out(&cgs);
    println!("===");
    let s = super::second::start(cgs.funcs);
    use super::bf::*;
    use super::stack::*;
    let stream = s
        .iter()
        .map(|x| convert(x.clone()))
        .collect::<Vec<StackInst>>();

    dbg!(&stream);
    let transpilation = translate(&stream);
    println!("{}", show_bf(&transpilation, cfg!(feature = "debugbf")));

    println!("\nExecution stack:\n");
    exec_stack_program(&stream);
    println!("\nExecution bf:\n");

    exec_bf(&transpilation);

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
                _ => unreachable!(),
            }, // 二項演算子
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
        }
    }
}

fn first_out(cgs: &CodeGenStatus) {
    for f in cgs.funcs.iter() {
        println!("{}:", f.sig.symbol.ident.get_name());
        for o in f.body.iter() {
            print!(" ");
            match o {
                StackCommand::Push(this) => println!("Push {}", this.oneline()),
                StackCommand::BinaryOP(this) => println!("{:?}", this),
                StackCommand::Symbol(this) => println!("Symbol {}", this.oneline()),
                StackCommand::Alloc(this) => println!("Alloca {}", this.to_rust_format()),
                StackCommand::Store => println!("{:?}", o), //　計算結果が下　対象は上
                StackCommand::Load(ty) => println!("Load {:?}", ty.to_rust_format()),
                StackCommand::Label(this) => println!("Label {:?}", this), // ラベル定義
                StackCommand::Goto(this) => println!("Jump {:?}", this),
                StackCommand::Call(_) => println!("{:?}", o),
                StackCommand::Return => println!("{:?}", o),
                StackCommand::ReturnPoint(_) => println!("{:?}", o),
                StackCommand::Branch(a, b) => println!("Branch ({:?},{:?})", a, b),
                StackCommand::FramePop => println!("{:?}", o),
                StackCommand::Name(s) => println!("Name {}", s.oneline()),
                StackCommand::IndexAccess(ty) => println!("IndexAccess {}", ty.to_rust_format()),
                StackCommand::SellOut => println!("{:?}", o),
                StackCommand::Comment(this) => println!("Comment {}", this),
                StackCommand::GlobalAddress => println!("{:?}", o),
                StackCommand::Address => println!("{:?}", o),
                StackCommand::AcsessUseGa => println!("{:?}", o),
                StackCommand::AcsessUseLa => println!("{:?}", o),
            }
        }
    }
}
