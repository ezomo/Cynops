use crate::ast::*;
use std::collections::HashMap;

// CodeGenStatus の定義
pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub variables: HashMap<Ident, String>,
    pub return_value_ptr: Option<String>,
    pub return_label: Option<String>,
    pub break_labels: Vec<String>,    // break用ラベルのスタック
    pub continue_labels: Vec<String>, // continue用ラベルのスタック
    pub string_literals: HashMap<String, String>, // 文字列リテラルのキャッシュ
    pub global_counter: usize,        // グローバル変数用カウンタ
    pub label_counter: usize,         // ラベル用カウンタ
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
            string_literals: HashMap::new(),
            global_counter: 0,
            label_counter: 0,
        }
    }

    pub fn push_loop_labels(&mut self, break_label: String, continue_label: String) {
        self.break_labels.push(break_label);
        self.continue_labels.push(continue_label);
    }

    pub fn pop_loop_labels(&mut self) {
        self.break_labels.pop();
        self.continue_labels.pop();
    }

    pub fn current_break_label(&self) -> Option<&String> {
        self.break_labels.last()
    }

    pub fn current_continue_label(&self) -> Option<&String> {
        self.continue_labels.last()
    }

    pub fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn get_or_create_string_literal(&mut self, s: &str) -> String {
        if let Some(existing) = self.string_literals.get(s) {
            existing.clone()
        } else {
            let global_name = format!("str_{}", self.global_counter);
            self.global_counter += 1;

            // グローバル文字列定数を宣言
            println!(
                "@{} = private unnamed_addr constant [{}x i8] c\"{}\\00\"",
                global_name,
                s.len() + 1,
                s
            );

            self.string_literals
                .insert(s.to_string(), global_name.clone());
            global_name
        }
    }
}

pub struct NameGenerator {
    counter: usize,
}

impl NameGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    fn next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn value(&mut self) -> String {
        format!("%{}", self.next())
    }

    // pub fn lavel(&mut self) -> String {
    //     format!("{}", self.next())
    // }
}

pub trait ToLLVMIR {
    fn to_llvmir(&self) -> &str;
}

impl Ident {
    pub fn get_name(&self) -> &str {
        &self.name
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
            Self::Minus => "sub",    // 0 - x として実装
            Self::Bang => "icmp eq", // x == 0 として実装
            Self::Tilde => "xor",    // x ^ -1 として実装
            _ => "unknown",
        }
    }
}

pub fn i1toi64(name_i1: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.value();
    println!("{} = zext i1 {} to i64", name, name_i1);
    name
}

pub fn i64toi1(name_i64: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.value();
    println!("{} = icmp ne i64 {}, 0", name, name_i64);
    name
}

impl Type {
    pub fn get_llvm_type(&self) -> String {
        match self {
            Type::Void => "void".to_string(),
            Type::Int => "i64".to_string(),
            Type::Double => "double".to_string(),
            Type::Char => "i8".to_string(),
            Type::Pointer(_) => "ptr".to_string(),
            Type::Array(arr) => {
                format!("[{} x {}]", arr.length, &arr.array_of.get_llvm_type())
            }
            _ => todo!("未対応の型: {:?}", self),
        }
    }
}
