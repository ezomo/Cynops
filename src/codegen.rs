use crate::ast::*;
use std::collections::HashMap;

// CodeGenStatus の定義
pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub variables: HashMap<Ident, String>,
    pub return_value_ptr: Option<String>,
    pub return_label: Option<String>,
    pub arrays: HashMap<Ident, (String, usize)>, // 配列名 -> (ポインタ, サイズ)
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
            arrays: HashMap::new(),
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
            Self::Caret => "xor",
            Self::Pipe => "or",
            Self::Ampersand => "and",
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
            Self::Minus => "sub",
            Self::Bang => "icmp eq", // !x は x == 0 と同等
            Self::Tilde => "xor",    // ~x は x xor -1
            _ => "add",              // その他はデフォルト
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

fn gen_logical_and(lhs: Expr, rhs: Expr, cgs: &mut CodeGenStatus) -> String {
    let label_num = cgs.name_gen.next();
    let lhs_val = gen_expr(lhs, cgs);
    let lhs_bool = i32toi1(lhs_val, cgs);

    println!(
        "br i1 %{}, label %and_rhs{}, label %and_end{}",
        lhs_bool, label_num, label_num
    );

    println!("and_rhs{}:", label_num);
    let rhs_val = gen_expr(rhs, cgs);
    let rhs_bool = i32toi1(rhs_val, cgs);
    println!("br label %and_end{}", label_num);

    println!("and_end{}:", label_num);
    let result = cgs.name_gen.next();
    println!(
        "%{} = phi i1 [false, %and_rhs{}], [%{}, %and_end{}]",
        result, label_num, rhs_bool, label_num
    );

    i1toi32(result, cgs)
}

fn gen_logical_or(lhs: Expr, rhs: Expr, cgs: &mut CodeGenStatus) -> String {
    let label_num = cgs.name_gen.next();
    let lhs_val = gen_expr(lhs, cgs);
    let lhs_bool = i32toi1(lhs_val, cgs);

    println!(
        "br i1 %{}, label %or_end{}, label %or_rhs{}",
        lhs_bool, label_num, label_num
    );

    println!("or_rhs{}:", label_num);
    let rhs_val = gen_expr(rhs, cgs);
    let rhs_bool = i32toi1(rhs_val, cgs);
    println!("br label %or_end{}", label_num);

    println!("or_end{}:", label_num);
    let result = cgs.name_gen.next();
    println!(
        "%{} = phi i1 [true, %or_end{}], [%{}, %or_rhs{}]",
        result, label_num, rhs_bool, label_num
    );

    i1toi32(result, cgs)
}

fn gen_compound_assign(op: AssignOp, lhs: Expr, rhs: Expr, cgs: &mut CodeGenStatus) -> String {
    if let Expr::Ident(ident) = lhs {
        let rhs_val = gen_expr(rhs, cgs);
        let ptr = cgs.variables.get(&ident).unwrap();

        // 現在の値を読み込み
        let current = cgs.name_gen.next();
        println!("%{} = load i32, ptr %{}", current, ptr);

        // 演算を実行
        let result = cgs.name_gen.next();
        let op_str = match op {
            AssignOp::PlusEqual => "add",
            AssignOp::MinusEqual => "sub",
            AssignOp::AsteriskEqual => "mul",
            AssignOp::SlashEqual => "sdiv",
            AssignOp::PercentEqual => "srem",
            AssignOp::CaretEqual => "xor",
            AssignOp::PipeEqual => "or",
            AssignOp::AmpersandEqual => "and",
            AssignOp::LessLessEqual => "shl",
            AssignOp::GreaterGreaterEqual => "ashr",
            _ => "add",
        };

        println!("%{} = {} i32 %{}, %{}", result, op_str, current, rhs_val);
        println!("store i32 %{}, ptr %{}", result, ptr);

        return result;
    } else {
        panic!("The left side is not variable!");
    }
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
            BinaryOp::Logical(logical) => match logical {
                Logical::AmpersandAmpersand => gen_logical_and(*binary.lhs, *binary.rhs, cgs),
                Logical::PipePipe => gen_logical_or(*binary.lhs, *binary.rhs, cgs),
            },
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
            _ => gen_compound_assign(assign.op, *assign.lhs, *assign.rhs, cgs),
        },
        Expr::Unary(unary) => match unary.op {
            UnaryOp::Minus => {
                let operand = gen_expr(*unary.expr, cgs);
                let name = cgs.name_gen.next();
                println!("%{} = sub i32 0, %{}", name, operand);
                return name;
            }
            UnaryOp::Bang => {
                let operand = gen_expr(*unary.expr, cgs);
                let bool_val = i32toi1(operand, cgs);
                let name = cgs.name_gen.next();
                println!("%{} = xor i1 %{}, true", name, bool_val);
                i1toi32(name, cgs)
            }
            UnaryOp::Tilde => {
                let operand = gen_expr(*unary.expr, cgs);
                let name = cgs.name_gen.next();
                println!("%{} = xor i32 %{}, -1", name, operand);
                return name;
            }
            UnaryOp::Ampersand => {
                // アドレス取得 (&x)
                if let Expr::Ident(ident) = *unary.expr {
                    let ptr = cgs.variables.get(&ident).unwrap();
                    let name = cgs.name_gen.next();
                    println!("%{} = ptrtoint ptr %{} to i32", name, ptr);
                    return name;
                } else {
                    panic!("Cannot take address of non-variable!");
                }
            }
            UnaryOp::Asterisk => {
                // ポインタ参照 (*ptr)
                let ptr_val = gen_expr(*unary.expr, cgs);
                let ptr = cgs.name_gen.next();
                let result = cgs.name_gen.next();
                println!("%{} = inttoptr i32 %{} to ptr", ptr, ptr_val);
                println!("%{} = load i32, ptr %{}", result, ptr);
                return result;
            }
            UnaryOp::PlusPlus => {
                // 前置インクリメント (++x)
                if let Expr::Ident(ident) = *unary.expr {
                    let ptr = cgs.variables.get(&ident).unwrap();
                    let current = cgs.name_gen.next();
                    let result = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", current, ptr);
                    println!("%{} = add i32 %{}, 1", result, current);
                    println!("store i32 %{}, ptr %{}", result, ptr);
                    return result;
                } else {
                    panic!("Cannot increment non-variable!");
                }
            }
            UnaryOp::MinusMinus => {
                // 前置デクリメント (--x)
                if let Expr::Ident(ident) = *unary.expr {
                    let ptr = cgs.variables.get(&ident).unwrap();
                    let current = cgs.name_gen.next();
                    let result = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", current, ptr);
                    println!("%{} = sub i32 %{}, 1", result, current);
                    println!("store i32 %{}, ptr %{}", result, ptr);
                    return result;
                } else {
                    panic!("Cannot decrement non-variable!");
                }
            }
        },
        Expr::Postfix(postfix) => match postfix.op {
            PostfixOp::PlusPlus => {
                // 後置インクリメント (x++)
                if let Expr::Ident(ident) = *postfix.expr {
                    let ptr = cgs.variables.get(&ident).unwrap();
                    let current = cgs.name_gen.next();
                    let incremented = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", current, ptr);
                    println!("%{} = add i32 %{}, 1", incremented, current);
                    println!("store i32 %{}, ptr %{}", incremented, ptr);
                    return current; // 古い値を返す
                } else {
                    panic!("Cannot increment non-variable!");
                }
            }
            PostfixOp::MinusMinus => {
                // 後置デクリメント (x--)
                if let Expr::Ident(ident) = *postfix.expr {
                    let ptr = cgs.variables.get(&ident).unwrap();
                    let current = cgs.name_gen.next();
                    let decremented = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", current, ptr);
                    println!("%{} = sub i32 %{}, 1", decremented, current);
                    println!("store i32 %{}, ptr %{}", decremented, ptr);
                    return current; // 古い値を返す
                } else {
                    panic!("Cannot decrement non-variable!");
                }
            }
        },
        Expr::Subscript(subscript) => {
            // 配列アクセス (array[index])
            if let Expr::Ident(array_name) = *subscript.name {
                let index_val = gen_expr(*subscript.index, cgs);

                if let Some((array_ptr, _size)) = cgs.arrays.get(&array_name) {
                    let element_ptr = cgs.name_gen.next();
                    let result = cgs.name_gen.next();
                    println!(
                        "%{} = getelementptr i32, ptr %{}, i32 %{}",
                        element_ptr, array_ptr, index_val
                    );
                    println!("%{} = load i32, ptr %{}", result, element_ptr);
                    return result;
                } else {
                    panic!("Array not found: {}", array_name.get_name());
                }
            } else {
                panic!("Complex array access not supported yet!");
            }
        }
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
        Expr::Ternary(ternary) => {
            let cond_val = gen_expr(*ternary.cond, cgs);
            let cond_bool = i32toi1(cond_val, cgs);
            let label_num = cgs.name_gen.next();

            println!(
                "br i1 %{}, label %then{}, label %else{}",
                cond_bool, label_num, label_num
            );

            println!("then{}:", label_num);
            let then_val = gen_expr(*ternary.then_branch, cgs);
            println!("br label %end{}", label_num);

            println!("else{}:", label_num);
            let else_val = gen_expr(*ternary.else_branch, cgs);
            println!("br label %end{}", label_num);

            println!("end{}:", label_num);
            let result = cgs.name_gen.next();
            println!(
                "%{} = phi i32 [%{}, %then{}], [%{}, %else{}]",
                result, then_val, label_num, else_val, label_num
            );

            return result;
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
                for init in init_vec {
                    let ident = init.r.ident.clone();

                    // 型に基づいて適切な処理を行う
                    match init.r.ty {
                        Type::Array(array) => {
                            // 配列の宣言
                            let array_ptr = cgs.name_gen.next();
                            println!("%{} = alloca [{} x i32]", array_ptr, array.length);

                            if let Some(InitData::Compound(init_list)) = init.l {
                                // 配列初期化子 {1, 2, 3, ...}
                                for (i, init_data) in init_list.into_iter().enumerate() {
                                    if let InitData::Expr(expr) = init_data {
                                        let value = gen_expr(expr, cgs);
                                        let element_ptr = cgs.name_gen.next();
                                        println!(
                                            "%{} = getelementptr [{} x i32], ptr %{}, i32 0, i32 {}",
                                            element_ptr, array.length, array_ptr, i
                                        );
                                        println!("store i32 %{}, ptr %{}", value, element_ptr);
                                    }
                                }
                            } else {
                                // デフォルト初期化（ゼロ初期化）
                                println!(
                                    "call void @llvm.memset.p0.i32(ptr %{}, i8 0, i32 {}, i1 false)",
                                    array_ptr,
                                    array.length * 4
                                );
                            }

                            cgs.arrays.insert(ident, (array_ptr.clone(), array.length));
                            return array_ptr;
                        }
                        Type::Pointer(_) => {
                            // ポインタの宣言
                            let ptr = cgs.name_gen.next();
                            println!("%{} = alloca ptr", ptr);

                            if let Some(InitData::Expr(expr)) = init.l {
                                let value = gen_expr(expr, cgs);
                                let converted_ptr = cgs.name_gen.next();
                                println!("%{} = inttoptr i32 %{} to ptr", converted_ptr, value);
                                println!("store ptr %{}, ptr %{}", converted_ptr, ptr);
                            }

                            cgs.variables.insert(ident, ptr.clone());
                            return ptr;
                        }
                        _ => {
                            // 通常の変数宣言
                            let ptr = cgs.name_gen.next();
                            println!("%{} = alloca i32", ptr);

                            if let Some(InitData::Expr(expr)) = init.l {
                                let rhs = gen_expr(expr, cgs);
                                println!("store i32 %{}, ptr %{}", rhs, ptr);
                            }

                            cgs.variables.insert(ident, ptr.clone());
                            return ptr;
                        }
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
    cgs.arrays.clear();

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
    // LLVM内組み関数の宣言
    println!("declare void @llvm.memset.p0.i32(ptr, i8, i32, i1)");

    for item in program.items {
        gen_top_level(item, cgs);
    }
    IGNORE.to_string()
}
