use std::collections::HashMap;

use crate::setting::*;
#[allow(unused_imports)]
use crate::string2tree::*;

trait ToLLVMIR {
    fn to_llvmir(&self) -> &str;
}

impl ToLLVMIR for Arithmetic {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "sdiv",
        }
    }
}
impl ToLLVMIR for Comparison {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Eq => "icmp eq",
            Self::Neq => "icmp ne",
            Self::Lt => "icmp slt",
            Self::Le => "icmp sle",
            Self::Gt => "icmp sgt",
            Self::Ge => "icmp sge",
        }
    }
}
pub struct TmpNameGen {
    counter: usize,
}

impl TmpNameGen {
    pub fn new() -> Self {
        TmpNameGen { counter: 0 }
    }

    pub fn next(&mut self) -> String {
        let name = format!("%tmp{}", self.counter);
        self.counter += 1;
        name
    }
}

pub fn generate(
    mut node: Box<Node>,
    name_gen: &mut TmpNameGen,
    variables: &mut HashMap<String, String>,
) -> String {
    match node.token {
        Token::Symbol(symbol) => {
            match symbol {
                Symbol::Arithmetic(ari) => {
                    let lhs = generate(node.lhs.take().unwrap(), name_gen, variables);
                    let rhs = generate(node.rhs.take().unwrap(), name_gen, variables);
                    let name1 = name_gen.next();

                    println!("  {} = {} i32 {}, {}", name1, ari.to_llvmir(), lhs, rhs);
                    return name1;
                }
                Symbol::Comparison(com) => {
                    let lhs = generate(node.lhs.take().unwrap(), name_gen, variables);
                    let rhs = generate(node.rhs.take().unwrap(), name_gen, variables);
                    let name1 = name_gen.next();

                    let name2 = name_gen.next();
                    println!("  {} = {} i32 {}, {}", name1, com.to_llvmir(), lhs, rhs);
                    println!("  {} = zext i1 {} to i32", name2, name1);
                    return name2;
                }
                Symbol::Assignment => {
                    // lhs は ident なので、もう一度解析する必要あり
                    if let Token::Ident(ref idn) = node.lhs.as_ref().unwrap().token {
                        let rhs = generate(node.rhs.take().unwrap(), name_gen, variables);
                        let ptr = variables.entry(idn.clone()).or_insert_with(|| {
                            let alloc = name_gen.next();
                            println!("  {} = alloca i32", alloc);
                            alloc
                        });
                        println!("  store i32 {}, i32* {}", rhs, ptr);
                        return ptr.clone();
                    } else {
                        panic!("The left side is not variable!");
                    }
                }
                Symbol::Return => {
                    let lhs = generate(node.lhs.take().unwrap(), name_gen, variables);
                    println!("  ret i32 {}", lhs);

                    return "finished".to_string();
                }
                _ => panic!(),
            }
        }
        Token::Number(num) => {
            let name1 = name_gen.next();
            println!("  {} = add i32 0, {}", name1, num);
            return name1;
        }
        Token::Ident(ref idn) => {
            if let Some(ptr) = variables.get(idn) {
                // 既にallcoされた変数
                let tmp = name_gen.next();
                println!("  {} = load i32, i32* {}", tmp, ptr);
                return tmp;
            } else {
                // 初めて出てきた変数
                let ptr = name_gen.next();
                println!("  {} = alloca i32", ptr);
                variables.insert(idn.clone(), ptr.clone());
                return ptr;
            }
        }
    }
}

#[test]
fn test() {
    let a = "a = 5; return a;";
    let mut b = tokenize(&a.to_string());
    let ast = program(&mut b);
    let mut name_gen = TmpNameGen::new();
    let mut hashmap = HashMap::new();
    for i in &ast {
        i.print_ast();
        generate(i.clone(), &mut name_gen, &mut hashmap);
    }
}
