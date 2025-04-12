use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    let value: i32 = match args[1].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("引数は整数である必要があります");
            process::exit(1);
        }
    };

    println!("; ModuleID = 'main'");
    println!("declare i32 @printf(i8*, ...)");
    println!();
    println!("define i32 @main() {{");
    println!("  ret i32 {}", value);
    println!("}}");
}
