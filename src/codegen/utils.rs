use crate::op::*;
use crate::sema::ast::*;
use core::str;
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
    GlobalAddress,                  //グローバルアドレスをスタックに乗せる
    Address,                        //変数のグローバルアドレスをスタックに
    AcsessUseGa,                    //メンバアクセス グローバルアドレスできるようにする
    AcsessUseLa,                    //メンバアクセス グローバルアドレスできるようにする
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
            StackCommand::Address => write!(f, "Address"),
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

#[derive(Debug)]
pub struct CodeGenSpace {
    pub variables: HashMap<Ident, String>,
}

impl CodeGenSpace {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
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

    pub fn register_variable(&self, sybmol: Symbol, string: String) {
        sybmol
            .scope
            .ptr
            .upgrade()
            .unwrap()
            .borrow_mut()
            .codege_space
            .variables
            .insert(sybmol.ident, string);
    }
}

pub struct NameGenerator {
    counter: usize,
}

#[derive(Debug, Clone, PartialEq)]

pub enum LLVMType {
    GrobalConst,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LLVMValue {
    pub variable: String,
    pub ty: LLVMType,
}

impl ToString for LLVMValue {
    fn to_string(&self) -> String {
        self.variable.clone()
    }
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

    pub fn global_const(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("@{}", self.next()),
            ty: LLVMType::GrobalConst,
        }
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

impl Type {
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub fn is_address(&self) -> bool {
        match self {
            Type::Void => false,
            Type::DotDotDot => false,
            Type::Int => false,
            Type::Double => false,
            Type::Char => false,
            Type::Pointer(_) => true,
            Type::Array(_) => true,
            Type::Func(_) => true,
            Type::Struct(_) => false,
            Type::Enum(_) => false,
            Type::Union(_) => false,
            Type::Typedef(_) => false,
            _ => unreachable!(),
        }
    }

    pub fn to_llvm_format(&self) -> String {
        match self {
            Type::Void => "void".to_string(),
            Type::DotDotDot => "...".to_string(),
            Type::Int => "i32".to_string(),
            Type::Double => "double".to_string(),
            Type::Char => "i8".to_string(),
            Type::Pointer(ty) => {
                format!("{}*", ty.to_llvm_format())
            }
            Type::Array(arr) => {
                format!(
                    "[{} x {}]",
                    arr.length.as_ref().unwrap().consume_const(),
                    &arr.array_of.to_llvm_format()
                )
            }
            Type::Func(func) => {
                format!(
                    "{} ({})",
                    func.return_type.to_llvm_format(),
                    func.params
                        .iter()
                        .filter(|x| !x.is_void())
                        .map(|x| x.to_llvm_format())
                        .collect::<Vec<String>>()
                        .join(","),
                )
            }
            Type::Struct(this) => format!("%{}", this.symbol.ident.to_string()),
            Type::Enum(_) => Type::Int.to_llvm_format(),
            Type::Union(this) => format!("%{}", this.symbol.ident.to_string()),
            Type::Typedef(this) => this.get_type().unwrap().to_llvm_format(),
            _ => todo!("未対応の型: {:?}", self),
        }
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
