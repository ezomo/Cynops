use std::collections::HashMap;
use std::usize;

use super::StackCommand;
use super::utils::SFunc;
use super::{SLabel, SLabelReserved};
use crate::codegen::r#type::Size;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

type Address = usize;

#[derive(Debug, Clone)]
pub enum SeStackCommand {
    Push(usize),              // スタックに値を乗せる
    Branch(Address, Address), //True ,False
    BinaryOP(BinaryOp),       // 二項演算子
    UnaryOp(UnaryOp),         //単演算子
    Alloc(Address),           //型のサイズだけメモリ確保
    DeAlloc(Address),         //型のサイズだけメモリ確保
    WriteAddr,                // stackの一番上の値をアドレスに書き込み
    ReadAddr,                 //計算結果が下　対象までのstack上の距離
    Label(Address),           // ラベル定義
    Goto,
    Comment(String),
    Exit,
    Copy,
    SellOut,

    Input,
}

impl From<SLabel> for Address {
    fn from(this: SLabel) -> Self {
        this.0
    }
}

#[derive(Debug, Clone)]
pub struct CodeGenStatus {
    pub outpus: Vec<SeStackCommand>,
    pub grobal_address: usize,
    pub stack_size_func: usize,
    pub alloced: Vec<usize>,
    pub symbol_table: HashMap<Symbol, Address>,
}

impl CodeGenStatus {
    fn new() -> Self {
        Self {
            outpus: Vec::default(),
            grobal_address: 0,
            stack_size_func: 0,
            alloced: vec![0],
            symbol_table: HashMap::new(),
        }
    }
    fn add_stck(&mut self, size: usize) {
        self.stack_size_func += size;
    }
    fn sub_stack(&mut self, size: usize) {
        self.stack_size_func -= size;
    }

    fn head_sack_func(&self) -> usize {
        self.stack_size_func
    }
    fn head_sack_func_reset(&mut self) {
        self.stack_size_func = 0;
    }

    fn add_alloc(&mut self, size: usize) {
        *self.alloced.last_mut().unwrap() += size;
    }
    fn reset_alloc(&mut self) {
        self.alloced = vec![0];
    }

    fn sub_alloc(&mut self, size: usize) {
        *self.alloced.last_mut().unwrap() -= size;
    }
}

pub fn start(inputs: Vec<SFunc>) -> Vec<SeStackCommand> {
    let mut cgs = CodeGenStatus::new();

    let mut entry: Option<Symbol> = None;
    for func in &inputs {
        if func.sig.symbol.ident == "main".into() {
            entry = Some(func.sig.symbol.clone());
        }
        cgs.symbol_table
            .insert(func.sig.symbol.clone(), func.entry.into());
    }

    // エントリーポイント設定
    {
        cgs.outpus
            .insert(0, SeStackCommand::Label(SLabelReserved::Entry as usize));
        cgs.push_label(SLabelReserved::Exit.into());
        cgs.push_grobal(0); // Grobal address for main
        cgs.push_label(SLabel(cgs.symbol_table[&entry.unwrap()]));
        cgs.outpus.push(SeStackCommand::Goto);
        cgs.sub_stack(1);
    }

    for func in inputs {
        cgs.head_sack_func_reset();
        cgs.reset_alloc();
        let palam_size = func
            .param_names
            .iter()
            .filter(|x| !x.get_type().unwrap().is_void())
            .count(); //一旦型は無視
        cgs.outpus
            .push(SeStackCommand::Comment(func.sig.symbol.ident.name.clone()));
        cgs.outpus.push(SeStackCommand::Label(func.entry.into()));

        {
            {
                cgs.add_stck(1); // Grobal address  分
                // ここが基準なので0
                cgs.grobal_address = cgs.head_sack_func();
            }
            func.param_names
                .iter()
                .filter(|x| !x.get_type().unwrap().is_void())
                .for_each(|x| {
                    // 順序固定　Symbol＋Nameの順序を維持
                    cgs.add_stck(1);
                    cgs.symbol_table.insert(x.clone(), cgs.head_sack_func());
                });
        }

        for cmd in func.body {
            match cmd {
                StackCommand::Push(TypedExpr { expr, .. }) => cgs.push_expr(expr),
                StackCommand::Input => {
                    cgs.outpus.push(SeStackCommand::Input);
                    cgs.add_stck(1);
                }
                StackCommand::BinaryOP(binary_op) => {
                    cgs.outpus.push(SeStackCommand::BinaryOP(binary_op));
                    cgs.sub_stack(1);
                }
                StackCommand::UnaryOp(unary_op) => {
                    cgs.outpus.push(SeStackCommand::UnaryOp(unary_op))
                }

                StackCommand::Symbol(symbol) => cgs.push_usize(cgs.symbol_table[&symbol]),
                StackCommand::Name(symbol) => {
                    // つまり配列の場合は先頭のアドレスが一番下になる．
                    _ = cgs.symbol_table.insert(symbol, cgs.head_sack_func())
                }
                StackCommand::Alloc(ty) => cgs.alloc(&ty),
                StackCommand::Store => {
                    // cgs.acsess();
                    cgs.outpus.push(SeStackCommand::WriteAddr);
                    cgs.sub_stack(2);

                    // 一下のアドレス，その下の実値
                }
                StackCommand::Load(ty) => cgs.load(ty),
                StackCommand::IndexAccess(ty) => {
                    cgs.mul(ty.size() as isize);
                    cgs.add();
                } // 下のアドレスから型とオフセットを使ってアドレス計算
                StackCommand::Label(this) => cgs.outpus.push(SeStackCommand::Label(this.into())),
                StackCommand::Goto(this) => {
                    cgs.outpus.push(SeStackCommand::Push(this.into()));
                    cgs.outpus.push(SeStackCommand::Goto);
                }
                StackCommand::Branch(label_true, label_false) => {
                    cgs.outpus.push(SeStackCommand::Branch(
                        label_true.into(),
                        label_false.into(),
                    ));
                    cgs.sub_stack(1);
                }
                StackCommand::Call(ty) => cgs.call(ty),
                StackCommand::Return => {
                    cgs.outpus
                        .push(SeStackCommand::Push(cgs.head_sack_func() + 1));
                    cgs.outpus.push(SeStackCommand::WriteAddr);
                    cgs.sub_stack(1); //writeaddrでアドレスを2消費する
                }
                StackCommand::FramePop => {
                    let delete = cgs.alloced.iter().sum::<usize>() + palam_size + 1;
                    cgs.outpus.push(SeStackCommand::DeAlloc(delete));
                    cgs.sub_stack(delete);
                    // +1は継承したgrobal address分

                    cgs.outpus.push(SeStackCommand::Goto);
                    //存在するだけで呼び出されていない関数もある．
                }
                StackCommand::ReturnPoint(repo) => cgs.push_label(repo),
                StackCommand::SellOut => {
                    cgs.outpus.push(SeStackCommand::SellOut);
                    cgs.sub_stack(1);
                }

                StackCommand::Comment(com) => cgs.outpus.push(SeStackCommand::Comment(com)),
                StackCommand::GlobalAddress => {
                    cgs.outpus
                        .push(SeStackCommand::Comment("push_global_address_start".into()));

                    {
                        cgs.load_grobal_address();
                        cgs.outpus
                            .push(SeStackCommand::Push(cgs.head_sack_func() - 1));
                        // 自分自身は新たな1となるので-1
                        cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                    }

                    cgs.outpus
                        .push(SeStackCommand::Comment("push_global_address_end".into()));
                }
                StackCommand::Address => {
                    cgs.load_grobal_address();
                    cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                    cgs.sub_stack(1);
                }
                StackCommand::AcsessUseGa => {
                    cgs.load_grobal_address();
                    cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::minus()));
                    cgs.sub_stack(1);
                    cgs.acsess();
                }
                StackCommand::AcsessUseLa => cgs.acsess(),
                StackCommand::BlockStart => {
                    cgs.alloced.push(0);
                }
                StackCommand::BlockEnd => {
                    let dealloc_size = cgs.alloced.pop().unwrap();
                    cgs.outpus.push(SeStackCommand::DeAlloc(dealloc_size));
                    cgs.sub_stack(dealloc_size);
                }
                StackCommand::Pop(ty) => {
                    cgs.outpus.push(SeStackCommand::Comment("Pop_start".into()));
                    let size = ty.size();
                    cgs.outpus.push(SeStackCommand::DeAlloc(size));
                    cgs.sub_stack(size);
                    cgs.outpus.push(SeStackCommand::Comment("Pop_end".into()));
                }
            }
        }

        // 関数が呼び出されていた場合は-1
    }

    {
        cgs.outpus
            .push(SeStackCommand::Label(SLabelReserved::Exit as usize));
        cgs.outpus.push(SeStackCommand::Exit);
    }

    clea_dedspace(&mut cgs.outpus);

    cgs.outpus
}

fn clea_dedspace(inputs: &mut Vec<SeStackCommand>) {
    let mut result = Vec::new();

    for window in inputs.windows(2) {
        result.push(window[0].clone());

        if matches!(window[0], SeStackCommand::Label(_))
            && let SeStackCommand::Label(ref second) = window[1]
        {
            result.push(SeStackCommand::Push(*second));
            result.push(SeStackCommand::Goto);
        }
    }

    // 最後の要素を忘れず追加
    if let Some(last) = inputs.last() {
        result.push(last.clone());
    }

    inputs.clear();
    inputs.extend(result);
}

impl CodeGenStatus {
    fn push_expr(&mut self, sema_expr: SemaExpr) {
        match sema_expr {
            SemaExpr::NumInt(this) => {
                self.outpus.push(SeStackCommand::Push(this));
                self.add_stck(1);
            }
            SemaExpr::Char(this) => {
                self.outpus.push(SeStackCommand::Push(this as usize));
                self.add_stck(1);
            }
            _ => unreachable!("{:?}", sema_expr.oneline()),
        }
    }
    fn push_label(&mut self, label: SLabel) {
        self.outpus.push(SeStackCommand::Push(label.into()));
        self.add_stck(1);
    }
    fn push_usize(&mut self, num: usize) {
        self.outpus.push(SeStackCommand::Push(num));
        self.add_stck(1);
    }
    fn push_isize(&mut self, num: isize) {
        if num < 0 {
            self.outpus.push(SeStackCommand::Push(0));
            self.outpus.push(SeStackCommand::Push(num.abs() as usize));
            self.outpus
                .push(SeStackCommand::BinaryOP(BinaryOp::minus()));
        } else {
            self.outpus.push(SeStackCommand::Push(num as usize));
        }
        self.add_stck(1);
    }
    fn push_grobal(&mut self, num: usize) {
        self.push_usize(num);
    }
    // stackの上に人にの整数定数をかける
    fn mul(&mut self, b: isize) {
        self.outpus
            .push(SeStackCommand::Comment("mul_start".into()));
        if b == 0 {
            self.outpus.push(SeStackCommand::Push(0));
            self.outpus.push(SeStackCommand::Push(1));
            self.outpus.push(SeStackCommand::WriteAddr);
        } else if b > 0 {
            (1..b).for_each(|_| self.outpus.push(SeStackCommand::Copy)); //すでに１つ乗っているので..=ではない
            (1..b).for_each(|_| self.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus())));
        } else {
            self.outpus.push(SeStackCommand::Copy);
            self.outpus.push(SeStackCommand::Push(0));
            self.outpus.push(SeStackCommand::Push(2));
            self.outpus.push(SeStackCommand::WriteAddr);
            (1..-b).for_each(|_| self.outpus.push(SeStackCommand::Copy)); //すでに１つ乗っているので..=ではない
            (1..-b).for_each(|_| self.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus())));
            self.outpus
                .push(SeStackCommand::BinaryOP(BinaryOp::minus()));
        }
        self.outpus.push(SeStackCommand::Comment("mul_end".into()));
    }

    fn add(&mut self) {
        self.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
        self.sub_stack(1);
    }

    // グローバルアドレスをローカルアドレスに変換してスタックに乗せる
    fn acsess(&mut self) {
        self.outpus
            .push(SeStackCommand::Push(self.head_sack_func() - 1));
        self.outpus
            .push(SeStackCommand::BinaryOP(BinaryOp::minus()));

        self.mul(-1);
    }

    fn alloc(&mut self, ty: &Type) {
        self.outpus.push(SeStackCommand::Alloc(ty.size()));
        self.add_stck(ty.size());
        self.add_alloc(ty.size());
    }

    fn load(&mut self, ty: Type) {
        let mut load_one = || {
            self.outpus.push(SeStackCommand::Push(1));
            self.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
            //こいつはpush分の計算がいる基準が違う
            self.outpus.push(SeStackCommand::ReadAddr);
            // 下のメモリを消費して上に積むからスタックサイズは変わらない
        };

        load_one();
    }

    fn load_grobal_address(&mut self) {
        self.outpus.push(SeStackCommand::Push(self.grobal_address));
        self.add_stck(1);
        self.acsess();
        self.load(Type::Int);
    }

    fn call(&mut self, ty: Type) {
        self.outpus.push(SeStackCommand::Goto);

        // println!("CallF {} {}", self.head_sack_all(), self.head_sack_func());
        // Goto消費
        self.sub_stack(1);

        // 返り値
        if !ty.as_func().unwrap().return_type.is_void() {
            // self.sub_stack(1);
            self.sub_alloc(1);
        }

        // 帰りアドレス分
        self.sub_stack(1);
        // 引数分

        self.sub_stack(
            ty.as_func()
                .unwrap()
                .params
                .iter()
                .filter(|x| !x.is_void())
                .count(),
        );

        // グローバルアドレス分
        self.sub_stack(1);

        // println!("CallE {} {}", self.head_sack_all(), self.head_sack_func());
    }
}
