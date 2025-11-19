use crate::op::*;
use crate::sema::ast::*;
use std::collections::HashMap;
use std::fmt::write;

#[derive(Debug, Clone, Copy)]
pub struct SLabel(pub usize);
#[derive(Clone)]
pub enum StackCommand {
    Comment(String),    // コメント
    Push(TypedExpr),    // スタックに値を乗せる
    BinaryOP(BinaryOp), // 二項演算子
    UnaryOp(UnaryOp),
    Symbol(Symbol),         //変数のアドレスをスタックに乗せる
    Name(Symbol),           // 変数名をスタックに乗せる下のAlloca命令と組み合わせて使う
    Alloc(Type),            //型のサイズだけメモリ確保
    Store,                  //　計算結果が下　対象は上
    Load(Type),             //下のメモリから値をロード
    IndexAccess(Type),      // 下のアドレスから型とオフセットを使ってアドレス計算
    Label(SLabel),          // ラベル定義
    Goto(SLabel),           // 無条件ジャンプ
    Branch(SLabel, SLabel), //True ,False
    Call(Type),             // 関数呼び出し 下の引数群を使う　アドレス＋引数群
    Return,                 // 関数からの復帰
    ReturnPoint(SLabel),    // 関数終了後の戻る場所
    FramePop,               // フレームを削除
    SellOut,                //一番上を出力
    GlobalAddress,          //グローバルアドレスをスタックに乗せる
    Address,                //変数のグローバルアドレスをスタックに
    AcsessUseGa,            //メンバアクセス グローバルアドレスできるようにする
    AcsessUseLa,            //メンバアクセス グローバルアドレスできるようにする
    Input,
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
            StackCommand::Store => write!(f, "Store"),
            StackCommand::Load(ty) => write!(f, "Load {}", ty.to_rust_format()),
            StackCommand::Label(this) => write!(f, "Label {:?}", this),
            StackCommand::Goto(this) => write!(f, "Jump {:?}", this),
            StackCommand::Call(this) => write!(f, "Call {:?}", this),
            StackCommand::Return => write!(f, "Return"),
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

pub fn load(fnc: impl Fn(TypedExpr, &mut CodeGenStatus), expr: TypedExpr, cgs: &mut CodeGenStatus) {
    let ty = expr.r#type.clone();
    fnc(expr, cgs);
    if cgs.is_left_val() && !ty.is_address() {
        cgs.outputs.push(StackCommand::Load(ty));
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

pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub outputs: Vec<StackCommand>,
    pub func_end: Option<SLabel>,
    pub funcs: Vec<SFunc>,
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
        }
    }

    pub fn is_left_val(&self) -> bool {
        if let Some(last) = self.outputs.last() {
            let typematch = |x: &SemaExpr| match x {
                SemaExpr::Assign(_) => false,
                SemaExpr::Binary(_) => false,
                SemaExpr::Call(_) => false,
                SemaExpr::Char(_) => false,
                SemaExpr::String(_) => false,
                SemaExpr::Symbol(_) => false,
                SemaExpr::NumInt(_) => false,
                SemaExpr::NumFloat(_) => false,
                SemaExpr::Subscript(_) => true,
                SemaExpr::MemberAccess(_) => true,
                SemaExpr::Ternary(_) => false,
                SemaExpr::Unary(_) => false,
                SemaExpr::Sizeof(_) => false,
                SemaExpr::Cast(_) => false,
                SemaExpr::Comma(_) => false,
            };
            match last {
                StackCommand::Push(te) => typematch(&te.r#expr),
                StackCommand::BinaryOP(_) => false,
                StackCommand::UnaryOp(_) => false,
                StackCommand::Symbol(_) => true,
                StackCommand::Alloc(_) => false,
                StackCommand::Store => false,
                StackCommand::Load(_) => true,
                StackCommand::Label(_) => false,
                StackCommand::Goto(_) => false,
                StackCommand::Call(_) => false, //多分？ pointer等の値を検討
                StackCommand::Return => false,  //多分？
                StackCommand::ReturnPoint(_) => false, //多分
                StackCommand::FramePop => false,
                StackCommand::Name(_) => false,
                StackCommand::IndexAccess(_) => true,
                StackCommand::Branch(_, _) => false,
                StackCommand::SellOut => false,
                StackCommand::Comment(_) => false,
                StackCommand::GlobalAddress => true,
                StackCommand::Address => true,
                StackCommand::AcsessUseGa => true, //怪しい
                StackCommand::AcsessUseLa => true, //怪しい
                StackCommand::Input => false,
            }
        } else {
            false
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
        Self { counter: 2 }
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
