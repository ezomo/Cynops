use crate::ast::*;
use crate::sema::TypedExpr;
use std::collections::HashMap;

// CodeGenStatus の定義
pub struct CodeGenStatus {
    pub name_gen: NameGenerator,
    pub variables: HashMap<Ident, String>,
    pub return_value_ptr: Option<String>,
    pub return_label: Option<String>,
    pub break_label: Option<String>,
    pub continue_label: Option<String>,
    pub string_literals: HashMap<String, String>, // 文字列リテラルのキャッシュ
    pub global_counter: usize,                    // グローバル変数用カウンタ
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
            break_label: None,
            continue_label: None,
            string_literals: HashMap::new(),
            global_counter: 0,
        }
    }

    pub fn push_loop_labels(&mut self, break_label: String, continue_label: String) {
        self.break_label = Some(break_label);
        self.continue_label = Some(continue_label);
    }

    pub fn pop_loop_labels(&mut self) {
        self.break_label = None;
        self.continue_label = None;
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

    pub fn next(&mut self) -> String {
        let name = format!("tmp{}", self.counter);
        self.counter += 1;
        name
    }

    pub fn next_with_prefix(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.counter);
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

fn i1toi32(name_i1: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = zext i1 %{} to i32", name, name_i1);
    name
}

fn i32toi1(name_i32: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = icmp ne i32 %{}, 0", name, name_i32);
    name
}

fn gen_typed_expr(expr: TypedExpr, cgs: &mut CodeGenStatus) -> String {
    gen_expr(expr.r#expr, cgs)
}

fn gen_expr(expr: crate::sema::SemaExpr, cgs: &mut CodeGenStatus) -> String {
    use crate::sema::SemaExpr;

    match expr {
        SemaExpr::Binary(binary) => match binary.op {
            BinaryOp::Arithmetic(ari) => {
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let rhs = gen_typed_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.next();

                println!("%{} = {} i32 %{}, %{}", name, ari.to_llvmir(), lhs, rhs);
                name
            }
            BinaryOp::Comparison(com) => {
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let rhs = gen_typed_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.next();

                println!("%{} = {} i32 %{}, %{}", name, com.to_llvmir(), lhs, rhs);
                i1toi32(name, cgs)
            }
            BinaryOp::Logical(Logical::AmpersandAmpersand) => {
                // 短絡評価: lhs && rhs
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let lhs_bool = i32toi1(lhs, cgs);

                let true_label = cgs.name_gen.next_with_prefix("and_true");
                let false_label = cgs.name_gen.next_with_prefix("and_false");
                let end_label = cgs.name_gen.next_with_prefix("and_end");

                println!(
                    "br i1 %{}, label %{}, label %{}",
                    lhs_bool, true_label, false_label
                );

                // true branch
                println!("{}:", true_label);
                let rhs = gen_typed_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label);

                // false branch
                println!("{}:", false_label);
                println!("br label %{}", end_label);

                // end
                println!("{}:", end_label);
                let result = cgs.name_gen.next();
                println!(
                    "%{} = phi i32 [%{}, %{}], [0, %{}]",
                    result, rhs, true_label, false_label
                );
                result
            }
            BinaryOp::Logical(Logical::PipePipe) => {
                // 短絡評価: lhs || rhs
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let lhs_bool = i32toi1(lhs.clone(), cgs);

                let false_label = cgs.name_gen.next_with_prefix("or_false");
                let true_label = cgs.name_gen.next_with_prefix("or_true");
                let end_label = cgs.name_gen.next_with_prefix("or_end");

                println!(
                    "br i1 %{}, label %{}, label %{}",
                    lhs_bool, true_label, false_label
                );

                // false branch
                println!("{}:", false_label);
                let rhs = gen_typed_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label);

                // true branch
                println!("{}:", true_label);
                println!("br label %{}", end_label);

                // end
                println!("{}:", end_label);
                let result = cgs.name_gen.next();
                println!(
                    "%{} = phi i32 [%{}, %{}], [%{}, %{}]",
                    result, lhs, true_label, rhs, false_label
                );
                result
            }
        },
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                if let SemaExpr::Ident(ident) = &assign.lhs.r#expr {
                    let rhs = gen_typed_expr(*assign.rhs, cgs);
                    let ptr = cgs.variables.get(ident).unwrap();
                    println!("store i32 %{}, ptr %{}", rhs, ptr);
                    rhs
                } else {
                    panic!("The left side is not variable!");
                }
            }
            _ => {
                // 複合代入演算子 (+=, -=, など)
                if let SemaExpr::Ident(ident) = &assign.lhs.r#expr {
                    let ptr = cgs.variables.get(ident).unwrap().clone();
                    let lhs_val = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", lhs_val, ptr);

                    let rhs = gen_typed_expr(*assign.rhs, cgs);
                    let result = cgs.name_gen.next();

                    let op = match assign.op {
                        AssignOp::PlusEqual => "add",
                        AssignOp::MinusEqual => "sub",
                        AssignOp::AsteriskEqual => "mul",
                        AssignOp::SlashEqual => "sdiv",
                        AssignOp::PercentEqual => "srem",
                        AssignOp::AmpersandEqual => "and",
                        AssignOp::PipeEqual => "or",
                        AssignOp::CaretEqual => "xor",
                        AssignOp::LessLessEqual => "shl",
                        AssignOp::GreaterGreaterEqual => "ashr",
                        _ => panic!("Unsupported assign op"),
                    };

                    println!("%{} = {} i32 %{}, %{}", result, op, lhs_val, rhs);
                    println!("store i32 %{}, ptr %{}", result, ptr);
                    result
                } else {
                    panic!("The left side is not variable!");
                }
            }
        },
        SemaExpr::NumInt(num) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = add i32 0, {}", name1, num);
            name1
        }
        SemaExpr::NumFloat(num) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = fadd double 0.0, {}", name1, num.0);
            name1
        }
        SemaExpr::Char(ch) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = add i8 0, {}", name1, ch as u8);
            name1
        }
        SemaExpr::String(s) => {
            let global_name = cgs.get_or_create_string_literal(&s);
            let name = cgs.name_gen.next();
            println!(
                "%{} = getelementptr inbounds [{}x i8], ptr @{}, i32 0, i32 0",
                name,
                s.len() + 1,
                global_name
            );
            name
        }
        SemaExpr::Ident(ident) => {
            let tmp = cgs.name_gen.next();
            let ptr = cgs.variables.get(&ident).unwrap();
            println!("%{} = load i32, ptr %{}", tmp, ptr);
            tmp
        }
        SemaExpr::Call(call) => {
            let name = cgs.name_gen.next();
            let args: Vec<String> = call
                .args
                .iter()
                .map(|arg| format!("i32 noundef %{}", gen_typed_expr(*arg.clone(), cgs)))
                .collect();

            let fn_name = match &call.func.r#expr {
                SemaExpr::Ident(idn) => idn.clone(),
                _ => panic!("Function call target is not an identifier"),
            };
            println!(
                "%{} = call i32 @{}({})",
                name,
                fn_name.get_name(),
                args.join(", ")
            );
            name
        }
        SemaExpr::Unary(unary) => {
            match unary.op {
                UnaryOp::Minus => {
                    let operand = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = sub i32 0, %{}", name, operand);
                    name
                }
                UnaryOp::Bang => {
                    let operand = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = icmp eq i32 %{}, 0", name, operand);
                    i1toi32(name, cgs)
                }
                UnaryOp::Tilde => {
                    let operand = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = xor i32 %{}, -1", name, operand);
                    name
                }
                UnaryOp::PlusPlus => {
                    // 前置インクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.next();
                        println!("%{} = load i32, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = add i32 %{}, 1", new_val, old_val);
                        println!("store i32 %{}, ptr %{}", new_val, ptr);
                        new_val
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                UnaryOp::MinusMinus => {
                    // 前置デクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.next();
                        println!("%{} = load i32, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = sub i32 %{}, 1", new_val, old_val);
                        println!("store i32 %{}, ptr %{}", new_val, ptr);
                        new_val
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
                UnaryOp::Ampersand => {
                    // アドレス演算子 &x
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        ptr // 変数のポインタをそのまま返す
                    } else {
                        panic!("& can only be applied to lvalues");
                    }
                }
                UnaryOp::Asterisk => {
                    // 間接参照演算子 *x
                    let ptr = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", name, ptr);
                    name
                }
            }
        }
        SemaExpr::Postfix(postfix) => {
            match postfix.op {
                PostfixOp::PlusPlus => {
                    // 後置インクリメント
                    if let SemaExpr::Ident(ident) = &postfix.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.next();
                        println!("%{} = load i32, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = add i32 %{}, 1", new_val, old_val);
                        println!("store i32 %{}, ptr %{}", new_val, ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                PostfixOp::MinusMinus => {
                    // 後置デクリメント
                    if let SemaExpr::Ident(ident) = &postfix.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.next();
                        println!("%{} = load i32, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = sub i32 %{}, 1", new_val, old_val);
                        println!("store i32 %{}, ptr %{}", new_val, ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
            }
        }
        SemaExpr::Ternary(ternary) => {
            let cond = gen_typed_expr(*ternary.cond, cgs);
            let cond_bool = i32toi1(cond, cgs);

            let true_label = cgs.name_gen.next_with_prefix("ternary_true");
            let false_label = cgs.name_gen.next_with_prefix("ternary_false");
            let end_label = cgs.name_gen.next_with_prefix("ternary_end");

            println!(
                "br i1 %{}, label %{}, label %{}",
                cond_bool, true_label, false_label
            );

            // true branch
            println!("{}:", true_label);
            let true_val = gen_typed_expr(*ternary.then_branch, cgs);
            println!("br label %{}", end_label);

            // false branch
            println!("{}:", false_label);
            let false_val = gen_typed_expr(*ternary.else_branch, cgs);
            println!("br label %{}", end_label);

            // end
            println!("{}:", end_label);
            let result = cgs.name_gen.next();
            println!(
                "%{} = phi i32 [%{}, %{}], [%{}, %{}]",
                result, true_val, true_label, false_val, false_label
            );
            result
        }
        SemaExpr::Subscript(subscript) => {
            // 配列の添字演算子 arr[index]
            let arr_ptr = gen_typed_expr(*subscript.name, cgs);
            let index = gen_typed_expr(*subscript.index, cgs);
            let name = cgs.name_gen.next();
            println!(
                "%{} = getelementptr inbounds i32, ptr %{}, i32 %{}",
                name, arr_ptr, index
            );
            let result = cgs.name_gen.next();
            println!("%{} = load i32, ptr %{}", result, name);
            result
        }
        SemaExpr::MemberAccess(member_access) => {
            // 構造体メンバアクセス
            let base = gen_typed_expr(*member_access.base, cgs);
            match member_access.kind {
                MemberAccessOp::Dot => {
                    // obj.member
                    let name = cgs.name_gen.next();
                    println!(
                        "%{} = getelementptr inbounds %struct, ptr %{}, i32 0, i32 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", result, name);
                    result
                }
                MemberAccessOp::MinusGreater => {
                    // ptr->member
                    let name = cgs.name_gen.next();
                    println!(
                        "%{} = getelementptr inbounds %struct, ptr %{}, i32 0, i32 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.next();
                    println!("%{} = load i32, ptr %{}", result, name);
                    result
                }
            }
        }
        SemaExpr::Sizeof(_sizeof) => {
            // sizeof演算子 - 簡略化のため4（intのサイズ）を返す
            let name = cgs.name_gen.next();
            println!("%{} = add i32 0, 4", name);
            name
        }
        SemaExpr::Cast(cast) => {
            // キャスト演算子 (type)expr
            let expr_val = gen_typed_expr(*cast.expr, cgs);
            // 簡略化のため、実際の型変換は行わずそのまま返す
            expr_val
        }
        SemaExpr::Comma(comma) => {
            // カンマ演算子 - 最後の式の値を返す
            let mut last_val = IGNORE.to_string();
            for assign in comma.assigns {
                last_val = gen_typed_expr(assign, cgs);
            }
            last_val
        }
    }
}

fn gen_control(control: Control, cgs: &mut CodeGenStatus) -> String {
    match control {
        Control::If(be) => {
            let con = i32toi1(gen_typed_expr(*be.cond, cgs), cgs);
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
            if let Some(else_branch) = be.else_branch {
                gen_stmt(*else_branch, cgs);
            }
            println!("br label %end_label{}", if_name);

            println!("end_label{}:", if_name);

            IGNORE.to_string()
        }
        Control::While(be) => {
            let while_name = cgs.name_gen.next();
            let break_label = format!("end_label{}", while_name);
            let continue_label = format!("cond_label{}", while_name);

            cgs.push_loop_labels(break_label.clone(), continue_label.clone());

            println!("br label %{}", continue_label);
            println!("{}:", continue_label);

            let con = i32toi1(gen_typed_expr(*be.cond, cgs), cgs);

            println!(
                "br i1 %{}, label %body_label{}, label %{}",
                con, while_name, break_label
            );

            println!("body_label{}:", while_name);

            gen_stmt(*be.body, cgs);

            println!("br label %{}", continue_label);
            println!("{}:", break_label);

            cgs.pop_loop_labels();

            IGNORE.to_string()
        }
        Control::DoWhile(be) => {
            let do_while_name = cgs.name_gen.next();
            let break_label = format!("end_label{}", do_while_name);
            let continue_label = format!("cond_label{}", do_while_name);

            cgs.push_loop_labels(break_label.clone(), continue_label.clone());

            println!("br label %body_label{}", do_while_name);
            println!("body_label{}:", do_while_name);

            gen_stmt(*be.body, cgs);

            println!("{}:", continue_label);
            let con = i32toi1(gen_typed_expr(*be.cond, cgs), cgs);

            println!(
                "br i1 %{}, label %body_label{}, label %{}",
                con, do_while_name, break_label
            );

            println!("{}:", break_label);

            cgs.pop_loop_labels();

            IGNORE.to_string()
        }
        Control::For(be) => {
            let for_name = cgs.name_gen.next();
            let break_label = format!("end_label{}", for_name);
            let continue_label = format!("step_label{}", for_name);

            cgs.push_loop_labels(break_label.clone(), continue_label.clone());

            // 初期化
            if let Some(init) = be.init {
                gen_typed_expr(*init, cgs);
            }

            println!("br label %cond_label{}", for_name);
            println!("cond_label{}:", for_name);

            let con = if let Some(cond) = be.cond {
                i32toi1(gen_typed_expr(*cond, cgs), cgs)
            } else {
                // 条件がない場合は常にtrue
                let name = cgs.name_gen.next();
                println!("%{} = add i1 0, true", name);
                name
            };

            println!(
                "br i1 %{}, label %body_label{}, label %{}",
                con, for_name, break_label
            );

            println!("body_label{}:", for_name);

            gen_stmt(*be.body, cgs);

            // step処理
            println!("{}:", continue_label);
            if let Some(step) = be.step {
                gen_typed_expr(*step, cgs);
            }

            println!("br label %cond_label{}", for_name);
            println!("{}:", break_label);

            cgs.pop_loop_labels();

            IGNORE.to_string()
        }
        Control::Switch(switch) => {
            let switch_name = cgs.name_gen.next();
            let break_label = format!("end_label{}", switch_name);

            cgs.push_loop_labels(break_label.clone(), IGNORE.to_string());

            let cond = gen_typed_expr(*switch.cond, cgs);

            // 各caseのラベルを生成
            let case_labels: Vec<String> = switch
                .cases
                .iter()
                .enumerate()
                .map(|(i, _)| format!("case_label{}_{}", switch_name, i))
                .collect();

            let default_label = format!("default_label{}", switch_name);

            // switch文のディスパッチ部分
            for (i, case) in switch.cases.iter().enumerate() {
                match case {
                    SwitchCase::Case(c) => {
                        let case_val = gen_typed_expr(c.expr.clone(), cgs);
                        let cmp_name = cgs.name_gen.next();
                        println!("%{} = icmp eq i32 %{}, %{}", cmp_name, cond, case_val);

                        let next_label = if i + 1 < case_labels.len() {
                            format!("check_case{}_{}", switch_name, i + 1)
                        } else {
                            default_label.clone()
                        };

                        println!(
                            "br i1 %{}, label %{}, label %{}",
                            cmp_name, case_labels[i], next_label
                        );
                    }
                    SwitchCase::Default(_) => {}
                }
            }

            // 各caseの処理
            for (i, case) in switch.cases.iter().enumerate() {
                match case {
                    SwitchCase::Case(c) => {
                        println!("{}:", case_labels[i]);
                        for stmt in &c.stmts {
                            gen_stmt(*stmt.clone(), cgs);
                        }
                    }
                    SwitchCase::Default(d) => {
                        println!("{}:", default_label);
                        for stmt in &d.stmts {
                            gen_stmt(*stmt.clone(), cgs);
                        }
                    }
                }
            }

            println!("{}:", break_label);

            cgs.pop_loop_labels();

            IGNORE.to_string()
        }
    }
}

fn gen_stmt(stmt: Stmt, cgs: &mut CodeGenStatus) -> String {
    match stmt {
        Stmt::Return(be) => {
            let rhs = if let Some(value) = be.value {
                gen_typed_expr(*value, cgs)
            } else {
                // voidの場合は0を返す
                let name = cgs.name_gen.next();
                println!("%{} = add i32 0, 0", name);
                name
            };

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
        Stmt::TypedExprStmt(expr) => gen_typed_expr(expr, cgs),
        Stmt::DeclStmt(decl) => match decl {
            DeclStmt::InitVec(init_vec) => {
                for init in init_vec {
                    let ident = init.r.ident.clone();
                    let ptr = cgs.name_gen.next();
                    println!("%{} = alloca i32", ptr);

                    if let Some(init_data) = init.l {
                        match init_data {
                            InitData::Expr(expr) => {
                                let rhs = gen_typed_expr(expr, cgs);
                                println!("store i32 %{}, ptr %{}", rhs, ptr);
                            }
                            InitData::Compound(_) => {
                                // 複合初期化子は未対応のため、デフォルト値0で初期化
                                println!("store i32 0, ptr %{}", ptr);
                            }
                        }
                    } else {
                        // 初期化なしの場合は0で初期化
                        println!("store i32 0, ptr %{}", ptr);
                    }

                    cgs.variables.insert(ident, ptr);
                }
                IGNORE.to_string()
            }
            DeclStmt::Struct(_) | DeclStmt::Union(_) | DeclStmt::Enum(_) | DeclStmt::Typedef(_) => {
                // 構造体、共用体、列挙型、typedefは未対応
                IGNORE.to_string()
            }
        },
        Stmt::Block(block) => {
            for stmt in block.statements {
                gen_stmt(*stmt, cgs);
            }
            IGNORE.to_string()
        }
        Stmt::Break => {
            if let Some(ref break_label) = cgs.break_label.clone() {
                println!("br label %{}", break_label);
            } else {
                panic!("Break statement outside of loop");
            }
            EVIL.to_string()
        }
        Stmt::Continue => {
            if let Some(ref continue_label) = cgs.continue_label.clone() {
                println!("br label %{}", continue_label);
            } else {
                panic!("Continue statement outside of loop");
            }
            EVIL.to_string()
        }
        Stmt::Goto(goto) => {
            println!("br label %{}", goto.label.get_name());
            EVIL.to_string()
        }
        Stmt::Label(label) => {
            println!("{}:", label.name.get_name());
            gen_stmt(*label.stmt, cgs)
        }
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
