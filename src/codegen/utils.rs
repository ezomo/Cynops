use crate::op::*;
use crate::sema::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct SLabel(usize);

#[derive(Debug)]
pub enum StackCommand {
    Push(TypedExpr),
    BinaryOP(BinaryOp),
    Symbol(Symbol),
    Alloca(Symbol),
    Store, //　計算結果が下　対象は上
    Load,
    Pop,
    Label(SLabel),       // ラベル定義
    Jump(SLabel),        // 無条件ジャンプ
    JumpIfFalse(SLabel), // スタックトップがfalseならジャンプ
    Call,
}

pub struct Func {
    pub sig: FunctionSig,
    pub param_names: Vec<Ident>,
    pub body: Block,
}

pub fn load(fnc: impl Fn(TypedExpr, &mut CodeGenStatus), expr: TypedExpr, cgs: &mut CodeGenStatus) {
    fnc(expr, cgs);
    if cgs.is_left_val() {
        cgs.outpus.push(StackCommand::Load);
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
    pub return_value_ptr: Option<String>,
    pub return_label: Option<LLVMValue>,
    pub break_labels: Vec<LLVMValue>,    // break用ラベルのスタック
    pub continue_labels: Vec<LLVMValue>, // continue用ラベルのスタック
    pub outpus: Vec<StackCommand>,
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
            return_value_ptr: None,
            return_label: None,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            outpus: Vec::new(),
        }
    }

    pub fn is_left_val(&self) -> bool {
        if let Some(last) = self.outpus.last() {
            match last {
                StackCommand::Push(te) => match &te.expr {
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
                },
                StackCommand::BinaryOP(_) => false,
                StackCommand::Symbol(_) => true,
                StackCommand::Alloca(_) => false,
                StackCommand::Store => false,
                StackCommand::Load => false,
                StackCommand::Pop => false,
                StackCommand::Label(_) => false,
                StackCommand::Jump(_) => false,
                StackCommand::JumpIfFalse(_) => false,
                StackCommand::Call => false, //多分？
            }
        } else {
            false
        }
    }

    pub fn current_break_label(&self) -> Option<&LLVMValue> {
        self.break_labels.last()
    }

    pub fn current_continue_label(&self) -> Option<&LLVMValue> {
        self.continue_labels.last()
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

    pub fn get_variable(&self, sybmol: &Symbol) -> Option<String> {
        sybmol
            .scope
            .ptr
            .upgrade()
            .unwrap()
            .borrow()
            .codege_space
            .variables
            .get(&sybmol.ident)
            .cloned()
    }
}

pub struct NameGenerator {
    counter: usize,
}

#[derive(Debug, Clone, PartialEq)]

pub enum LLVMType {
    GrobalConst,
    Register,
    Label,
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
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    fn next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn register(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("%tmp{}", self.next()),
            ty: LLVMType::Register,
        }
    }

    pub fn global_const(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("@{}", self.next()),
            ty: LLVMType::GrobalConst,
        }
    }

    pub fn label(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("label_{}", self.next()),
            ty: LLVMType::Label,
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

    pub fn get_fnc_name(&self) -> String {
        format!("@{}", &self.name)
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
