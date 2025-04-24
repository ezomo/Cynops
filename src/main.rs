use std::collections::HashMap;
use std::env;
use std::process;
mod setting;

mod tree2code;
use tree2code::*;

mod string2tree;
use string2tree::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }
    println!("; ModuleID = 'main'");
    println!("define i32 @main() {{");

    let mut b = tokenize(&args[1].to_string());
    let ast = program(&mut b);
    let mut name_gen = TmpNameGen::new();
    let mut hashmap = HashMap::new();
    for i in &ast {
        generate(i.clone(), &mut name_gen, &mut hashmap);
    }
    println!("}}")
}
