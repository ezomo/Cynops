use crate::setting::*;
#[allow(unused_imports)]
use crate::string2tree::*;

pub fn generate(node: Box<Node>, id_counter: &mut usize) -> String {
    match node.token {
        Token::Number(n) => {
            let name = format!("%tmp{}", *id_counter);
            println!("  {} = add i32 0, {}", name, n);
            *id_counter += 1;
            return name;
        }
        Token::Symbol(sym) => {
            let lhs = generate(node.lhs.unwrap(), id_counter);
            let rhs = generate(node.rhs.unwrap(), id_counter);
            let name = format!("%tmp{}", *id_counter);
            *id_counter += 1;

            let op = match sym {
                Symbol::Arithmetic(Arithmetic::Add) => "add".to_string(),
                Symbol::Arithmetic(Arithmetic::Sub) => "sub".to_string(),
                Symbol::Arithmetic(Arithmetic::Mul) => "mul".to_string(),
                Symbol::Arithmetic(Arithmetic::Div) => "sdiv".to_string(),
                Symbol::Comparison(Comparison::Eq) => "icmp eq".to_string(),
                Symbol::Comparison(Comparison::Neq) => "icmp ne".to_string(),
                Symbol::Comparison(Comparison::Lt) => "icmp slt".to_string(),
                Symbol::Comparison(Comparison::Le) => "icmp sle".to_string(),
                Symbol::Comparison(Comparison::Gt) => "icmp sgt".to_string(),
                Symbol::Comparison(Comparison::Ge) => "icmp sge".to_string(),
                _ => panic!("error"),
            };

            println!("  {} = {} i32 {}, {}", name, op, lhs, rhs);

            if matches!(sym, Symbol::Comparison(_)) {
                let name_1 = format!("%tmp{}", *id_counter);
                *id_counter += 1;
                println!("  {} = zext i1 {} to i32", name_1, name);
                return name_1;
            }
            return name;
        }
        _ => todo!(),
    }
}

#[test]
fn test() {
    let a = "1;1+2;a*(b+c);";
    let mut b = tokenize(&a.to_string());
    println!("{:?}", b);
    let ast = program(&mut b);
    for i in &ast {
        i.print_ast();
    }
}
