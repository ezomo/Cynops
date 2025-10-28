use std::collections::HashMap;
use std::usize;

use super::SLabel;
use super::StackCommand;
use super::utils::SFunc;
use crate::op::*;
use crate::sema::ast::*;
use crate::visualize::OneLine;

type Address = usize;

#[derive(Debug, Clone)]
pub enum SeStackCommand {
    Push(usize),              // スタックに値を乗せる
    Branch(Address, Address), //True ,False
    BinaryOP(BinaryOp),       // 二項演算子
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
}

impl From<SLabel> for Address {
    fn from(this: SLabel) -> Self {
        this.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodeGenStatus {
    pub outpus: Vec<SeStackCommand>,
    pub stack_size_all: usize,
    pub stack_size_func: usize,
    pub alloced: usize,
    pub symbol_table: HashMap<Symbol, Address>,
}

impl CodeGenStatus {
    fn add_stck(&mut self, size: usize) {
        self.stack_size_all += size;
        self.stack_size_func += size;
    }
    fn sub_stack(&mut self, size: usize) {
        self.stack_size_all -= size;
        self.stack_size_func -= size;
    }
    fn sub_stack_all(&mut self, size: usize) {
        self.stack_size_all -= size;
    }
    fn head_sack_all(&self) -> usize {
        self.stack_size_all
    }
    fn head_sack_func(&self) -> usize {
        self.stack_size_func
    }
    fn head_sack_func_reset(&mut self) {
        self.stack_size_func = 0;
    }

    fn add_alloc(&mut self, size: usize) {
        self.alloced += size;
    }
    fn reset_alloc(&mut self) {
        self.alloced = 0;
    }

    fn sub_alloc(&mut self, size: usize) {
        self.alloced -= size;
    }
}

pub fn start(inputs: Vec<SFunc>) -> Vec<SeStackCommand> {
    let mut cgs = CodeGenStatus::default();

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
        cgs.outpus.insert(0, SeStackCommand::Label(1));
        cgs.push_label(SLabel(2));
        cgs.push_label(SLabel(cgs.symbol_table[&entry.unwrap()]));
        cgs.outpus.insert(3, SeStackCommand::Goto);
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
            func.param_names
                .iter()
                .filter(|x| !x.get_type().unwrap().is_void())
                .for_each(|x| {
                    cgs.add_stck(1);
                    cgs.symbol_table.insert(x.clone(), cgs.head_sack_func());
                });
        }

        for cmd in func.body {
            match cmd {
                StackCommand::Push(TypedExpr { expr, .. }) => cgs.push_expr(expr),
                StackCommand::BinaryOP(binary_op) => {
                    cgs.outpus.push(SeStackCommand::BinaryOP(binary_op));
                    cgs.sub_stack(1);
                }
                StackCommand::Symbol(symbol) => match symbol.get_type().unwrap() {
                    Type::Func(_) => cgs.push_usize(cgs.symbol_table[&symbol]),
                    _ => {
                        cgs.push_usize(cgs.symbol_table[&symbol]);
                    }
                },
                StackCommand::Name(symbol) => {
                    _ = cgs.symbol_table.insert(symbol, cgs.head_sack_func())
                }
                StackCommand::Alloc(ty) => cgs.alloc(&ty),
                StackCommand::Store => {
                    cgs.acsess();
                    cgs.outpus.push(SeStackCommand::WriteAddr);
                    cgs.sub_stack(2);

                    // 一下のアドレス，その下の実値
                }
                StackCommand::Load(ty) => {
                    cgs.acsess();
                    match ty {
                        Type::Pointer(p) => match *p {
                            Type::Func(_) => {
                                cgs.outpus.push(SeStackCommand::Push(1));
                                cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                                //こいつはpush分の計算がいる基準が違う
                                cgs.outpus.push(SeStackCommand::ReadAddr);
                                // 下のメモリを消費して上に積むからスタックサイズは変わらないcgs
                                cgs.outpus
                                    .push(SeStackCommand::Comment("↑関数のアドレス".into()))
                            }

                            _ => {
                                cgs.outpus.push(SeStackCommand::Comment("p_s".into()));
                                cgs.outpus.push(SeStackCommand::Copy);
                                cgs.outpus.push(SeStackCommand::Push(2));
                                cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                                cgs.outpus.push(SeStackCommand::ReadAddr);
                                cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                                cgs.outpus.push(SeStackCommand::Comment("p_e".into()));

                                // 下のメモリを消費して上に積むからスタックサイズは変わらない
                            }
                        },
                        _ => {
                            cgs.outpus.push(SeStackCommand::Push(1));
                            cgs.outpus.push(SeStackCommand::BinaryOP(BinaryOp::plus()));
                            //こいつはpush分の計算がいる基準が違う
                            cgs.outpus.push(SeStackCommand::ReadAddr);
                            // 下のメモリを消費して上に積むからスタックサイズは変わらない
                        }
                    }
                } //下のメモリから値をロード
                StackCommand::IndexAccess(Type) => {} // 下のアドレスから型とオフセットを使ってアドレス計算
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

                    println!("{},{}", cgs.head_sack_all(), cgs.head_sack_func());
                    cgs.sub_stack(1); //writeaddrでアドレスを2消費する
                    println!("{},{}", cgs.head_sack_all(), cgs.head_sack_func());
                }
                StackCommand::FramePop => {
                    cgs.outpus
                        .push(SeStackCommand::DeAlloc(cgs.alloced + palam_size));
                    // println!("Alloc_{} {}", cgs.head_sack_all(), cgs.head_sack_func());
                    // println!("{}+ {}", cgs.alloced, palam_size);
                    cgs.sub_stack(cgs.alloced + palam_size);
                    // println!("DeAlloc_{} {}", cgs.head_sack_all(), cgs.head_sack_func());

                    cgs.outpus.push(SeStackCommand::Goto);
                    //存在するだけで呼び出されていない関数もある．
                }
                StackCommand::ReturnPoint(repo) => cgs.push_label(repo),
                StackCommand::SellOut => {
                    cgs.outpus.push(SeStackCommand::SellOut);
                    cgs.sub_stack(1);
                }
            }
        }

        // 関数が呼び出されていた場合は-1
        if cgs.stack_size_all > 1 {
            cgs.sub_stack_all(1);
        }
    }

    {
        cgs.outpus.push(SeStackCommand::Label(2));
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
            _ => unreachable!(),
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

    fn acsess(&mut self) {
        self.outpus
            .push(SeStackCommand::Push(self.head_sack_func() - 1));
        self.outpus
            .push(SeStackCommand::BinaryOP(BinaryOp::minus()));

        self.mul(-1);
    }

    fn alloc(&mut self, _ty: &Type) {
        self.outpus.push(SeStackCommand::Alloc(1));
        self.add_stck(1);
        self.add_alloc(1);
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
        // println!("CallE {} {}", self.head_sack_all(), self.head_sack_func());
    }
}
