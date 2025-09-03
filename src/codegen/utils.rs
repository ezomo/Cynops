use crate::op::*;
use crate::sema::ast::*;
use std::collections::HashMap;

// CodeGenStatus の定義
pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub variables: HashMap<Ident, String>,
    pub return_value_ptr: Option<String>,
    pub return_label: Option<LLVMValue>,
    pub break_labels: Vec<LLVMValue>,    // break用ラベルのスタック
    pub continue_labels: Vec<LLVMValue>, // continue用ラベルのスタック
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
            variables: HashMap::new(),
            return_value_ptr: None,
            return_label: None,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
        }
    }

    pub fn push_loop_labels(&mut self, break_label: LLVMValue, continue_label: LLVMValue) {
        self.break_labels.push(break_label);
        self.continue_labels.push(continue_label);
    }

    pub fn pop_loop_labels(&mut self) {
        self.break_labels.pop();
        self.continue_labels.pop();
    }

    pub fn current_break_label(&self) -> Option<&LLVMValue> {
        self.break_labels.last()
    }

    pub fn current_continue_label(&self) -> Option<&LLVMValue> {
        self.continue_labels.last()
    }
}

pub struct NameGenerator {
    counter: usize,
}

#[derive(Debug, Clone, PartialEq)]

pub enum LLVMType {
    Const,
    Register,
    Variable,
    Label,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LLVMValue {
    pub variable: String,
    pub ty: LLVMType,
}

impl LLVMValue {
    pub fn new<T: ToString>(variable: T, ty: LLVMType) -> Self {
        Self {
            variable: variable.to_string(),
            ty,
        }
    }
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
            variable: format!("%{}", self.next()),
            ty: LLVMType::Register,
        }
    }

    pub fn void(&mut self) -> LLVMValue {
        LLVMValue {
            variable: "void".to_string(),
            ty: LLVMType::Void,
        }
    }

    pub fn variable(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("%{}", self.next()),
            ty: LLVMType::Variable,
        }
    }

    pub fn label(&mut self) -> LLVMValue {
        LLVMValue {
            variable: format!("label_{}", self.next()),
            ty: LLVMType::Label,
        }
    }
}

pub trait ToLLVMIR {
    fn to_llvmir(&self) -> &str;
}

impl Ident {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_fnc_name(&self) -> String {
        format!("@{}", &self.name)
    }
}

impl ToLLVMIR for Arithmetic {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Plus => "add",
            Self::Minus => "sub",
            Self::Asterisk => "mul",
            Self::Slash => "sdiv",
            Self::Percent => "srem",
            Self::Ampersand => "and",
            Self::Pipe => "or",
            Self::Caret => "xor",
            Self::LessLess => "shl",
            Self::GreaterGreater => "ashr",
        }
    }
}

impl ToLLVMIR for Comparison {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::EqualEqual => "icmp eq",
            Self::NotEqual => "icmp ne",
            Self::Less => "icmp slt",
            Self::LessEqual => "icmp sle",
            Self::Greater => "icmp sgt",
            Self::GreaterEqual => "icmp sge",
        }
    }
}

impl ToLLVMIR for UnaryOp {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Bang => "icmp eq", // x == 0 として実装
            Self::Tilde => "xor",    // x ^ -1 として実装
            _ => "unknown",
        }
    }
}

impl LLVMValue {
    pub fn i1toi64(self: LLVMValue, cgs: &mut CodeGenStatus) -> LLVMValue {
        let name = cgs.name_gen.register();
        println!("{} = zext i1 {} to i64", name.to_string(), self.to_string());
        name
    }

    pub fn i64toi1(&self, cgs: &mut CodeGenStatus) -> LLVMValue {
        let name = cgs.name_gen.register();
        println!("{} = icmp ne i64 {}, 0", name.to_string(), self.to_string());
        name
    }
}

pub fn load(ty: &Type, data: LLVMValue, cgs: &mut CodeGenStatus) -> LLVMValue {
    match data.ty {
        LLVMType::Variable => {
            let name = cgs.name_gen.register();
            println!(
                "{} = load {}, {}* {}",
                name.to_string(),
                ty.to_llvm_format(),
                ty.to_llvm_format(),
                data.to_string()
            );
            name
        }

        _ => data,
    }
}

pub fn new_load(
    fnc: impl Fn(TypedExpr, &mut CodeGenStatus) -> LLVMValue,
    expr: TypedExpr,
    cgs: &mut CodeGenStatus,
) -> LLVMValue {
    load(&expr.r#type.clone(), fnc(expr, cgs), cgs)
}

pub fn wrap(ty: &Type, data: LLVMValue, cgs: &mut CodeGenStatus) -> LLVMValue {
    match data.ty {
        LLVMType::Variable => data,
        LLVMType::Void => data,
        _ => {
            let name = cgs.name_gen.variable();
            println!("{} = alloca {}", name.to_string(), ty.to_llvm_format());

            println!(
                "store {} {}, {}* {}",
                ty.to_llvm_format(),
                data.to_string(),
                ty.to_llvm_format(),
                name.to_string()
            );

            name
        }
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
            Type::Int => "i64".to_string(),
            Type::Double => "double".to_string(),
            Type::Char => "i8".to_string(),
            Type::Pointer(ty) => {
                format!("{}*", ty.to_llvm_format())
            }
            Type::Array(arr) => {
                format!(
                    "[{} x {}]",
                    arr.length.clone().unwrap().consume_const(),
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
            _ => todo!("未対応の型: {:?}", self),
        }
    }
}
