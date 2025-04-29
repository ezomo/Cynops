use crate::setting::{
    node::Node,
    token::{Arithmetic, Comparison, ExprSymbol, Value},
    *,
};
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

pub fn generate(node: Box<Node>, cgs: &mut CodeGenStatus) -> String {
    match *node {
        Node::Expr(be) => {
            match be.op {
                ExprSymbol::Arithmetic(ari) => {
                    let lhs = generate(be.lhs, cgs);
                    let rhs = generate(be.rhs, cgs);
                    let name1 = cgs.name_gen.next();

                    println!("  {} = {} i32 {}, {}", name1, ari.to_llvmir(), lhs, rhs);
                    return name1;
                }
                ExprSymbol::Comparison(com) => {
                    let lhs = generate(be.lhs, cgs);
                    let rhs = generate(be.rhs, cgs);
                    let name1 = cgs.name_gen.next();

                    let name2 = cgs.name_gen.next();
                    println!("  {} = {} i32 {}, {}", name1, com.to_llvmir(), lhs, rhs);
                    println!("  {} = zext i1 {} to i32", name2, name1);
                    return name2;
                }
                ExprSymbol::Assignment => {
                    // lhs は ident なので、もう一度解析する必要あり
                    if let Node::Value(Value::Ident(ref idn)) = *be.lhs {
                        let rhs = generate(be.rhs, cgs);
                        let ptr = cgs.variables.entry(idn.clone()).or_insert_with(|| {
                            let alloc = cgs.name_gen.next();
                            println!("  {} = alloca i32", alloc);
                            alloc
                        });
                        println!("  store i32 {}, i32* {}", rhs, ptr);
                        return ptr.clone();
                    } else {
                        panic!("The left side is not variable!");
                    }
                }
                _ => panic!(),
            }
        }
        Node::Control(con) => {
            match con {
                node::Control::Return(be) => {
                    let lhs = generate(be.value, cgs);
                    println!("  ret i32 {}", lhs);
                }
                _ => panic!(),
            }

            return "finished".to_string();
        }
        Node::Value(vaule) => {
            match vaule {
                Value::Number(num) => {
                    let name1 = cgs.name_gen.next();
                    println!("  {} = add i32 0, {}", name1, num);
                    return name1;
                }
                Value::Ident(idn) => {
                    if let Some(ptr) = cgs.variables.get(&idn) {
                        // 既にallcoされた変数
                        let tmp = cgs.name_gen.next();
                        println!("  {} = load i32, i32* {}", tmp, ptr);
                        return tmp;
                    } else {
                        // 初めて出てきた変数
                        let ptr = cgs.name_gen.next();
                        println!("  {} = alloca i32", ptr);
                        cgs.variables.insert(idn.clone(), ptr.clone());
                        return ptr;
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tokenize::tokenize;
    let a = "a = 5; return a;";
    let mut b = tokenize(&a.to_string());
    let ast = program(&mut b);
    let mut cgs = CodeGenStatus::new();
    for i in &ast {
        generate(i.clone(), &mut cgs);
    }
}
