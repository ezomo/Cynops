use crate::op::*;
use crate::sema::ast::*;
use core::str;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct SLabel(pub usize);

#[derive(EnumIter)]
pub enum SLabelReserved {
    Entry = 1,
    Exit = 2,
}

impl From<SLabelReserved> for SLabel {
    fn from(value: SLabelReserved) -> Self {
        SLabel(value as usize)
    }
}

#[derive(Clone)]
pub enum StackCommand {
    Comment(String),    // コメント
    Push(TypedExpr),    // スタックに値を乗せる
    BinaryOP(BinaryOp), // 二項演算子
    UnaryOp(UnaryOp),
    Symbol(Symbol),                 //変数のアドレスをスタックに乗せる
    Name(Symbol),                   // 変数名をスタックに乗せる下のAlloca命令と組み合わせて使う
    Alloc(Type),                    //型のサイズだけメモリ確保
    Store(Type),                    //　計算結果が下　対象は上
    Load(Type),                     //下のメモリから値をロード
    IndexAccess(Type),              // 下のアドレスから型とオフセットを使ってアドレス計算
    Label(SLabel),                  // ラベル定義
    Goto(SLabel),                   // 無条件ジャンプ
    Branch(SLabel, SLabel),         //True ,False
    Call(Type),                     // 関数呼び出し 下の引数群を使う　アドレス＋引数群
    Return(Type),                   // 関数からの戻り値
    ReturnPoint(SLabel),            // 関数終了後の戻る場所
    FramePop,                       // フレームを削除
    SellOut,                        //一番上を出力
    GlobalAddress,                  //グローバルアドレスをスタックに乗せる　関数呼び出し時に使用
    La2GaAddress,                   //ローカルアドレスをグローバルアドレスに変換してスタックに乗せる
    AcsessUseGa,                    //メンバアクセス グローバルアドレスで変数にアクセスする
    AcsessUseLa,                    //メンバアクセス ローカルアドレスで変数にアクセスする
    Input,                          //入力
    BlockStart(SLabel),             //ブロック開始 label　id としてのSlabel
    BlockEnd(SLabel),               //ブロック終了
    Pop(Type),                      //型のサイズだけスタックを削除
    ClearStackFrom(SLabel),         // Slabelまでのsatckを削除
    MemberAccess(Vec<Type>, usize), // メンバアクセス 型リストとメンバのインデックス
}

impl std::fmt::Debug for StackCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::visualize::OneLine;
        match self {
            StackCommand::Push(this) => write!(f, "Push {}", this.oneline()),
            StackCommand::BinaryOP(this) => write!(f, "BinaryOP {:?}", this),
            StackCommand::UnaryOp(this) => write!(f, "UnaryOp{:?}", this),
            StackCommand::Symbol(this) => write!(f, "Symbol {}", this.oneline()),
            StackCommand::Alloc(this) => write!(f, "Alloca {}", this.to_rust_format()),
            StackCommand::Store(this) => write!(f, "Store {}", this.to_rust_format()),
            StackCommand::Load(ty) => write!(f, "Load {}", ty.to_rust_format()),
            StackCommand::Label(this) => write!(f, "Label {:?}", this),
            StackCommand::Goto(this) => write!(f, "Jump {:?}", this),
            StackCommand::Call(this) => write!(f, "Call {:?}", this),
            StackCommand::Return(this) => write!(f, "Return {:?}", this),
            StackCommand::ReturnPoint(this) => write!(f, "ReturnPoint {:?}", this),
            StackCommand::Branch(a, b) => write!(f, "Branch ({:?}, {:?})", a, b),
            StackCommand::FramePop => write!(f, "FramePop"),
            StackCommand::Name(s) => write!(f, "Name {}", s.oneline()),
            StackCommand::IndexAccess(ty) => write!(f, "IndexAccess {}", ty.to_rust_format()),
            StackCommand::SellOut => write!(f, "SellOut"),
            StackCommand::Comment(this) => write!(f, "Comment {}", this),
            StackCommand::GlobalAddress => write!(f, "GlobalAddress"),
            StackCommand::La2GaAddress => write!(f, "La2GaAddress"),
            StackCommand::AcsessUseGa => write!(f, "AccessUseGa"),
            StackCommand::AcsessUseLa => write!(f, "AccessUseLa"),
            StackCommand::Input => write!(f, "Input"),
            StackCommand::BlockStart(this) => write!(f, "BlockStart {:?}", this),
            StackCommand::BlockEnd(this) => write!(f, "BlockEnd {:?}", this),
            StackCommand::Pop(ty) => write!(f, "Pop {}", ty.to_rust_format()),
            StackCommand::ClearStackFrom(this) => write!(f, "ClearStackFrom {:?}", this),
            StackCommand::MemberAccess(ty, id) => {
                write!(f, "MemberAccess (types: {:?}, id: {})", ty, id)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SFunc {
    pub sig: FunctionSig,
    pub param_names: Vec<Symbol>,
    pub body: Vec<StackCommand>,
    pub entry: SLabel,
}

impl SFunc {
    pub fn new(
        sig: FunctionSig,
        param_names: Vec<Symbol>,
        body: Vec<StackCommand>,
        entry: SLabel,
    ) -> Self {
        Self {
            sig,
            param_names,
            body,
            entry,
        }
    }
}

impl From<TypedExpr> for StackCommand {
    fn from(expr: TypedExpr) -> Self {
        StackCommand::Push(expr)
    }
}
impl From<BinaryOp> for StackCommand {
    fn from(this: BinaryOp) -> Self {
        StackCommand::BinaryOP(this)
    }
}
impl From<SLabel> for StackCommand {
    fn from(this: SLabel) -> Self {
        StackCommand::Label(this)
    }
}

use strum_macros::EnumString;

#[derive(EnumString, Debug, Eq, Hash, PartialEq)]
pub enum InsertFunction {
    #[strum(serialize = "Greater")]
    Greater,
    #[strum(serialize = "Less")]
    Less,
    #[strum(serialize = "LessEqual")]
    LessEqual,
    #[strum(serialize = "GreaterEqual")]
    GreaterEqual,
    #[strum(serialize = "Ternary")]
    Ternary,
    #[strum(serialize = "Slash")]
    Slash,
    #[strum(serialize = "Mod")]
    Mod,
    #[strum(serialize = "Not")]
    Not,
    #[strum(serialize = "Land")]
    Land,
    #[strum(serialize = "print_int")]
    PrintInt,
    #[strum(serialize = "print_double")]
    PrintDouble,
    #[strum(serialize = "InitDouble")]
    InitDouble,
    #[strum(serialize = "DoubleGreater")]
    DoubleGreater,
    #[strum(serialize = "DoubleLess")]
    DoubleLess,
    #[strum(serialize = "DoubleEqual")]
    DoubleEqual,
}

pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub outputs: Vec<StackCommand>,
    pub func_end: Option<SLabel>,
    pub funcs: Vec<SFunc>,
    pub break_stack: Vec<(SLabel, SLabel)>, // (delete from, goto)
    pub continue_stack: Vec<(SLabel, SLabel)>, // (delete from, goto)
    pub insert_function: HashMap<InsertFunction, Symbol>,
}

impl Block {
    pub fn into_vec(self) -> Vec<Box<Stmt>> {
        self.statements
    }
}

impl CodeGenStatus {
    pub fn new() -> Self {
        Self {
            name_gen: NameGenerator::new(),
            outputs: Vec::new(),
            func_end: None,
            funcs: Vec::new(),
            break_stack: Vec::new(),
            continue_stack: Vec::new(),
            insert_function: HashMap::new(),
        }
    }
}

pub struct NameGenerator {
    counter: usize,
}

impl NameGenerator {
    // 1はentry_point
    // 2はexit_point
    pub fn new() -> Self {
        Self {
            counter: SLabelReserved::iter().map(|v| v as usize).max().unwrap(),
        }
    }

    fn next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn slabel(&mut self) -> SLabel {
        SLabel(self.next())
    }
}

impl Ident {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl TypedExpr {
    pub fn consume_const(&self) -> isize {
        match &self.r#expr {
            SemaExpr::NumInt(n) => *n as isize,
            _ => unreachable!(),
        }
    }
}

impl From<usize> for TypedExpr {
    fn from(this: usize) -> Self {
        TypedExpr::new(Type::Int, SemaExpr::NumInt(this))
    }
}

pub fn frac_as_usize(x: OrderedFloat<f64>) -> usize {
    let f = x.into_inner().abs().fract(); // 絶対値を取って小数部だけ

    if f == 0.0 {
        return 0;
    }

    let mut scale = 1.0;
    let max_digits = 15; // f64 の限界程度

    for _ in 0..max_digits {
        let v = f * scale;
        if (v - v.round()).abs() < 1e-12 {
            // 整数になった
            return v.round() as usize;
        }
        scale *= 10.0;
    }

    // ここに来るのは、「全然ぴったり整数にならない」場合
    // とりあえず丸めて返す
    (f * scale).round() as usize
}

impl Type {
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }
}

impl From<Symbol> for TypedExpr {
    fn from(this: Symbol) -> Self {
        Self {
            r#type: this.get_type().unwrap(),
            expr: SemaExpr::Symbol(this),
        }
    }
}
