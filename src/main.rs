use std::env;
use std::process;
use std::usize;

mod setting;
use setting::*;

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
    let ast = expr(&mut b);

    let mut id_counter: usize = 0;
    generate(ast, &mut id_counter);
    println!("  ret i32 %tmp{}", id_counter - 1);
    println!("}}")
}
