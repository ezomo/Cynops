use crate::ast::*;
use std::collections::HashMap;

// CodeGenStatus の定義
pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub variables: HashMap<Ident, String>,
    pub return_value_ptr: Option<String>,
    pub return_label: Option<String>,
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

    pub fn next(&mut self) -> String {
        let name = format!("tmp{}", self.counter);
        self.counter += 1;
        name
    }
}

trait ToLLVMIR {
    fn to_llvmir(&self) -> &str;
}

impl Ident {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

const EVIL: &str = "evil";
const IGNORE: &str = "ignore";

impl ToLLVMIR for Arithmetic {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Plus => "add",
            Self::Minus => "sub",
            Self::Asterisk => "mul",
            Self::Slash => "sdiv",
            Self::Percent => "srem",
            _ => "add", // デフォルト値
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

fn i1toi32(name_i1: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = zext i1 %{} to i32", name, name_i1);
    return name;
}

fn i32toi1(name_i32: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = icmp ne i32 %{}, 0", name, name_i32);
    return name;
}

fn gen_expr(expr: Expr, cgs: &mut CodeGenStatus) -> String {
    match expr {
        Expr::Binary(binary) => match binary.op {
            BinaryOp::Arithmetic(ari) => {
                let lhs = gen_expr(*binary.lhs, cgs);
                let rhs = gen_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.next();

                println!("%{} = {} i32 %{}, %{}", name, ari.to_llvmir(), lhs, rhs);
                return name;
            }
            BinaryOp::Comparison(com) => {
                let lhs = gen_expr(*binary.lhs, cgs);
                let rhs = gen_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.next();

                println!("%{} = {} i32 %{}, %{}", name, com.to_llvmir(), lhs, rhs);
                i1toi32(name, cgs)
            }
            _ => todo!(),
        },
        Expr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                if let Expr::Ident(ident) = *assign.lhs {
                    let rhs = gen_expr(*assign.rhs, cgs);
                    let ptr = cgs.variables.get(&ident).unwrap();
                    println!("store i32 %{}, ptr %{}", rhs, ptr);
                    return ptr.clone();
                } else {
                    panic!("The left side is not variable!");
                }
            }
            _ => todo!(),
        },
        Expr::NumInt(num) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = add i32 0, {}", name1, num);
            return name1;
        }
        Expr::Ident(ident) => {
            let tmp = cgs.name_gen.next();
            let ptr = cgs.variables.get(&ident).unwrap();
            println!("%{} = load i32, ptr %{}", tmp, ptr);
            return tmp;
        }
        Expr::Call(call) => {
            let name = cgs.name_gen.next();
            let args: Vec<String> = call
                .args
                .iter()
                .map(|arg| format!("i32 noundef %{}", gen_expr(*arg.clone(), cgs)))
                .collect();

            let fn_name = match *call.func {
                Expr::Ident(ref idn) => idn.clone(),
                _ => panic!(""),
            };
            println!(
                "%{} = call i32 @{}({})",
                name,
                fn_name.get_name(),
                args.join(", ")
            );
            return name;
        }
        _ => {
            println!("{:?}", expr);
            IGNORE.to_string()
        }
    }
}

fn gen_control(control: Control, cgs: &mut CodeGenStatus) -> String {
    match control {
        Control::If(be) => {
            let con = i32toi1(gen_expr(*be.cond, cgs), cgs);
            let if_name = cgs.name_gen.next();

            println!(
                "br i1 %{}, label %then_label{}, label %else_label{}",
                con, if_name, if_name
            );

            // then branch
            println!("then_label{}:", if_name);
            gen_stmt(*be.then_branch, cgs);
            println!("br label %end_label{}", if_name);

            // else branch
            println!("else_label{}:", if_name);
            be.else_branch.map(|b| gen_stmt(*b, cgs));
            println!("br label %end_label{}", if_name);

            println!("end_label{}:", if_name);

            IGNORE.to_string()
        }
        Control::While(be) => {
            let while_name = cgs.name_gen.next();

            println!("br label %cond_label{}", while_name);
            println!("cond_label{}:", while_name);

            let con = i32toi1(gen_expr(*be.cond, cgs), cgs);

            println!(
                "br i1 %{}, label %body_label{}, label %end_label{}",
                con, while_name, while_name
            );

            println!("body_label{}:", while_name);

            gen_stmt(*be.body, cgs);

            println!("br label %cond_label{}", while_name);
            println!("end_label{}:", while_name);

            IGNORE.to_string()
        }
        Control::For(be) => {
            let for_name = cgs.name_gen.next();

            // 初期化
            be.init
                .map(|x| gen_expr(*x, cgs))
                .unwrap_or(IGNORE.to_string());

            println!("br label %cond_label{}", for_name);
            println!("cond_label{}:", for_name);

            let con = i32toi1(gen_expr(*be.cond.unwrap(), cgs), cgs);

            println!(
                "br i1 %{}, label %body_label{}, label %end_label{}",
                con, for_name, for_name
            );

            println!("body_label{}:", for_name);

            gen_stmt(*be.body, cgs);

            // 増分
            be.step
                .map(|x| gen_expr(*x, cgs))
                .unwrap_or(IGNORE.to_string());

            println!("br label %cond_label{}", for_name);
            println!("end_label{}:", for_name);

            IGNORE.to_string()
        }
        _ => todo!(),
    }
}

fn gen_stmt(stmt: Stmt, cgs: &mut CodeGenStatus) -> String {
    match stmt {
        Stmt::Return(be) => {
            let rhs = gen_expr(*be.value.unwrap(), cgs);

            // return値をreturn_value_ptrに保存
            if let Some(ref return_ptr) = cgs.return_value_ptr {
                println!("store i32 %{}, ptr %{}", rhs, return_ptr);
            }

            // return_labelにジャンプ
            if let Some(ref return_label) = cgs.return_label {
                println!("br label %{}", return_label);
            }

            return EVIL.to_string();
        }
        Stmt::Control(control) => gen_control(control, cgs),
        Stmt::ExprStmt(expr) => gen_expr(expr, cgs),
        Stmt::DeclStmt(decl) => match decl {
            DeclStmt::InitVec(init_vec) => {
                // 最初の初期化のみ処理（元のコードの動作を保持）
                if let Some(init) = init_vec.first() {
                    let init_data = init.l.as_ref().expect("Initialization data expected");
                    if let InitData::Expr(expr) = init_data {
                        let rhs = gen_expr(expr.clone(), cgs);
                        let ident = init.r.ident.clone();
                        let ptr = cgs.variables.entry(ident).or_insert_with(|| {
                            let alloc = cgs.name_gen.next();
                            println!("%{} = alloca i32", alloc);
                            alloc
                        });
                        println!("store i32 %{}, ptr %{}", rhs, ptr);
                        return ptr.clone();
                    }
                }
                IGNORE.to_string()
            }
            _ => todo!(),
        },
        Stmt::Block(block) => {
            for stmt in block.statements {
                gen_stmt(*stmt, cgs);
            }
            IGNORE.to_string()
        }
        _ => IGNORE.to_string(),
    }
}

fn gen_function(function: FunctionDef, cgs: &mut CodeGenStatus) -> String {
    let name = function.sig.ident.clone();
    let params = function.param_names.clone();
    let args: Vec<String> = params.iter().map(|_| cgs.name_gen.next()).collect();

    println!(
        "define i32 @{}({}) {{",
        name.get_name(),
        args.iter()
            .map(|x| format!("i32 noundef %{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // return用の変数とラベルを設定
    let return_ptr = cgs.name_gen.next();
    let return_label = "return_label".to_string();
    println!("%{} = alloca i32", return_ptr);

    cgs.return_value_ptr = Some(return_ptr.clone());
    cgs.return_label = Some(return_label.clone());

    // 引数の処理
    for (i, param_name) in params.iter().enumerate() {
        let ptr = cgs.name_gen.next();
        println!("%{} = alloca i32", ptr);
        println!("store i32 %{}, ptr %{}", args[i], ptr);
        cgs.variables.insert(param_name.clone(), ptr);
    }

    // 関数本体の処理
    for stmt in function.body.into_vec() {
        gen_stmt(*stmt, cgs);
    }

    // 常にreturn_labelにジャンプ（return文がない場合のため）
    println!("br label %{}", return_label);

    // return_labelとreturn処理
    println!("{}:", return_label);
    println!("%val = load i32, ptr %{}", return_ptr);
    println!("ret i32 %val");

    println!("}}");

    // 関数終了時にreturn関連の情報をクリア
    cgs.return_value_ptr = None;
    cgs.return_label = None;
    cgs.variables.clear();

    return IGNORE.to_string();
}

fn gen_top_level(top_level: TopLevel, cgs: &mut CodeGenStatus) -> String {
    match top_level {
        TopLevel::FunctionDef(function_def) => gen_function(function_def, cgs),
        TopLevel::Stmt(stmt) => gen_stmt(stmt, cgs),
        TopLevel::FunctionProto(_) => IGNORE.to_string(), // 関数プロトタイプは無視
    }
}

pub fn generate_program(program: Program, cgs: &mut CodeGenStatus) -> String {
    for item in program.items {
        gen_top_level(item, cgs);
    }
    IGNORE.to_string()
}
