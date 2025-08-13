use crate::ast::*;
use crate::sema::TypedExpr;
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

fn i1toi64(name_i1: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = zext i1 %{} to i64", name, name_i1);
    name
}

fn i64toi1(name_i64: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = icmp ne i64 %{}, 0", name, name_i64);
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

                println!("%{} = {} i64 %{}, %{}", name, ari.to_llvmir(), lhs, rhs);
                name
            }
            BinaryOp::Comparison(com) => {
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let rhs = gen_typed_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.next();

                println!("%{} = {} i64 %{}, %{}", name, com.to_llvmir(), lhs, rhs);
                name
            }
            BinaryOp::Logical(Logical::AmpersandAmpersand) => {
                // 短絡評価: lhs && rhs
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let lhs_bool = i64toi1(lhs, cgs);

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
                    "%{} = phi i64 [%{}, %{}], [0, %{}]",
                    result, rhs, true_label, false_label
                );
                result
            }
            BinaryOp::Logical(Logical::PipePipe) => {
                // 短絡評価: lhs || rhs
                let lhs = gen_typed_expr(*binary.lhs, cgs);
                let lhs_bool = i64toi1(lhs.clone(), cgs);

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
                    "%{} = phi i64 [%{}, %{}], [%{}, %{}]",
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
                    println!("store i64 %{}, ptr %{}", rhs, ptr);
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
                    println!("%{} = load i64, ptr %{}", lhs_val, ptr);

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

                    println!("%{} = {} i64 %{}, %{}", result, op, lhs_val, rhs);
                    println!("store i64 %{}, ptr %{}", result, ptr);
                    result
                } else {
                    panic!("The left side is not variable!");
                }
            }
        },
        SemaExpr::NumInt(num) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = add i64 0, {}", name1, num);
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
                "%{} = getelementptr inbounds [{}x i8], ptr @{}, i64 0, i64 0",
                name,
                s.len() + 1,
                global_name
            );
            name
        }
        SemaExpr::Ident(ident) => {
            let tmp = cgs.name_gen.next();
            let ptr = cgs.variables.get(&ident).unwrap();
            println!("%{} = load i64, ptr %{}", tmp, ptr);
            tmp
        }
        SemaExpr::Call(call) => {
            let name = cgs.name_gen.next();
            let args: Vec<String> = call
                .args
                .iter()
                .map(|arg| format!("i64 noundef %{}", gen_typed_expr(*arg.clone(), cgs)))
                .collect();

            let fn_name = match &call.func.r#expr {
                SemaExpr::Ident(idn) => idn.clone(),
                _ => panic!("Function call target is not an identifier"),
            };
            println!(
                "%{} = call i64 @{}({})",
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
                    println!("%{} = sub i64 0, %{}", name, operand);
                    name
                }
                UnaryOp::Bang => {
                    let operand = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = icmp eq i64 %{}, 0", name, operand);
                    i1toi64(name, cgs)
                }
                UnaryOp::Tilde => {
                    let operand = gen_typed_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.next();
                    println!("%{} = xor i64 %{}, -1", name, operand);
                    name
                }
                UnaryOp::PlusPlus => {
                    // 前置インクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.next();
                        println!("%{} = load i64, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = add i64 %{}, 1", new_val, old_val);
                        println!("store i64 %{}, ptr %{}", new_val, ptr);
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
                        println!("%{} = load i64, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = sub i64 %{}, 1", new_val, old_val);
                        println!("store i64 %{}, ptr %{}", new_val, ptr);
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
                    println!("%{} = load i64, ptr %{}", name, ptr);
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
                        println!("%{} = load i64, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = add i64 %{}, 1", new_val, old_val);
                        println!("store i64 %{}, ptr %{}", new_val, ptr);
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
                        println!("%{} = load i64, ptr %{}", old_val, ptr);
                        let new_val = cgs.name_gen.next();
                        println!("%{} = sub i64 %{}, 1", new_val, old_val);
                        println!("store i64 %{}, ptr %{}", new_val, ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
            }
        }
        SemaExpr::Ternary(ternary) => {
            let cond = gen_typed_expr(*ternary.cond, cgs);
            let cond_bool = i64toi1(cond, cgs);

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
                "%{} = phi i64 [%{}, %{}], [%{}, %{}]",
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
                "%{} = getelementptr inbounds i64, ptr %{}, i64 %{}",
                name, arr_ptr, index
            );
            let result = cgs.name_gen.next();
            println!("%{} = load i64, ptr %{}", result, name);
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
                        "%{} = getelementptr inbounds %struct, ptr %{}, i64 0, i64 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.next();
                    println!("%{} = load i64, ptr %{}", result, name);
                    result
                }
                MemberAccessOp::MinusGreater => {
                    // ptr->member
                    let name = cgs.name_gen.next();
                    println!(
                        "%{} = getelementptr inbounds %struct, ptr %{}, i64 0, i64 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.next();
                    println!("%{} = load i64, ptr %{}", result, name);
                    result
                }
            }
        }
        SemaExpr::Sizeof(_sizeof) => {
            // sizeof演算子 - 簡略化のため4（intのサイズ）を返す
            let name = cgs.name_gen.next();
            println!("%{} = add i64 0, 4", name);
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
            let mut last_val = "".to_string();
            for assign in comma.assigns {
                last_val = gen_typed_expr(assign, cgs);
            }
            last_val
        }
    }
}
mod gen_stmt {
    use crate::ast::*;
    use crate::codegen::CodeGenStatus;
    use crate::codegen::gen_expr;
    use crate::codegen::gen_typed_expr;
    use crate::sema::TypedExpr;

    pub fn stmt(stmt: Stmt, cgs: &mut CodeGenStatus) {
        match stmt {
            Stmt::Block(block) => self::block(block, cgs),
            Stmt::DeclStmt(declstmt) => self::declstmt(declstmt, cgs),
            Stmt::Control(control) => self::control(control, cgs),
            Stmt::Break => r#break(cgs),
            Stmt::Continue => r#continue(cgs),
            Stmt::Return(ret) => r#return(ret, cgs),
            Stmt::Goto(goto) => self::goto(goto, cgs),
            Stmt::Label(label) => self::label(label, cgs),
            Stmt::TypedExprStmt(expr) => {
                let _ = gen_typed_expr(expr, cgs);
            }
        }
    }

    pub fn block(block: Block, cgs: &mut CodeGenStatus) {
        for _stmt in block.into_vec() {
            stmt(*_stmt, cgs);
        }
    }

    fn declstmt(declstmt: DeclStmt, cgs: &mut CodeGenStatus) {
        match declstmt {
            DeclStmt::InitVec(inits) => {
                for init in inits {
                    declare_variable(init, cgs);
                }
            }
            _ => {
                // Struct, Union, Enum, Typedef は今回は対象外
                todo!("構造体、共用体、列挙型、typedef は未対応")
            }
        }
    }

    fn declare_variable(init: Init, cgs: &mut CodeGenStatus) {
        let var_name = cgs.name_gen.next_with_prefix("var");
        let llvm_type = get_llvm_type(&init.r.ty);

        // 変数を割り当て
        println!("  %{} = alloca {}", var_name, llvm_type);

        // 変数名をマップに登録
        cgs.variables.insert(init.r.ident.clone(), var_name.clone());

        // 初期化子がある場合は初期化
        if let Some(init_data) = init.l {
            initialize_variable(&var_name, init_data, &init.r.ty, cgs);
        }
    }

    fn initialize_variable(
        var_name: &str,
        init_data: InitData,
        var_type: &Type,
        cgs: &mut CodeGenStatus,
    ) {
        match init_data {
            InitData::Expr(typed_expr) => {
                // 単純な式による初期化
                let value = gen_typed_expr(typed_expr, cgs);
                let llvm_type = get_llvm_type(var_type);
                println!("  store {} %{}, ptr %{}", llvm_type, value, var_name);
            }
            InitData::Compound(compound_list) => {
                match var_type {
                    Type::Array(arr) => {
                        // 配列の初期化 {1, 2, 3}
                        for (index, element) in compound_list.into_iter().enumerate() {
                            let element_ptr = cgs.name_gen.next();
                            let array_type =
                                format!("[{} x {}]", arr.length, get_llvm_type(&arr.array_of));
                            println!(
                                "  %{} = getelementptr inbounds {}, ptr %{}, i64 0, i64 {}",
                                element_ptr, array_type, var_name, index
                            );

                            initialize_variable(&element_ptr, element, &arr.array_of, cgs);
                        }
                    }
                    _ => {
                        todo!("構造体・共用体の複合初期化は未対応")
                    }
                }
            }
        }
    }

    fn get_llvm_type(ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int => "i64".to_string(),
            Type::Double => "double".to_string(),
            Type::Char => "i8".to_string(),
            Type::Pointer(_) => "ptr".to_string(),
            Type::Array(arr) => {
                format!("[{} x {}]", arr.length, get_llvm_type(&arr.array_of))
            }
            _ => todo!("未対応の型: {:?}", ty),
        }
    }

    fn control(control: Control, cgs: &mut CodeGenStatus) {
        match control {
            Control::If(if_stmt) => controls::r#if(if_stmt, cgs),
            Control::While(while_stmt) => controls::r#while(while_stmt, cgs),
            Control::DoWhile(do_while_stmt) => controls::r#do_while(do_while_stmt, cgs),
            Control::For(for_stmt) => controls::r#for(for_stmt, cgs),
            Control::Switch(switch_stmt) => controls::r#switch(switch_stmt, cgs),
        }
    }

    fn r#break(cgs: &mut CodeGenStatus) {
        if let Some(break_label) = cgs.current_break_label() {
            println!("  br label %{}", break_label);
        } else {
            panic!("break statement outside of loop");
        }
    }

    fn r#continue(cgs: &mut CodeGenStatus) {
        if let Some(continue_label) = cgs.current_continue_label() {
            println!("  br label %{}", continue_label);
        } else {
            panic!("continue statement outside of loop");
        }
    }

    fn r#return(ret: Return, cgs: &mut CodeGenStatus) {
        let rhs = if let Some(value) = ret.value {
            gen_typed_expr(*value, cgs)
        } else {
            // voidの場合は0を返す
            let name = cgs.name_gen.next();
            println!("%{} = add i64 0, 0", name);
            name
        };

        // return値をreturn_value_ptrに保存
        if let Some(ref return_ptr) = cgs.return_value_ptr {
            println!("store i64 %{}, ptr %{}", rhs, return_ptr);
        }

        // return_labelにジャンプ
        if let Some(ref return_label) = cgs.return_label {
            println!("br label %{}", return_label);
        }
    }

    fn goto(goto: Goto, _cgs: &mut CodeGenStatus) {
        println!("  br label %{}", goto.label.get_name());
    }

    fn label(label: Label, cgs: &mut CodeGenStatus) {
        println!("{}:", label.name.get_name());
        stmt(*label.stmt, cgs);
    }

    mod controls {
        use super::*;

        pub fn r#if(if_stmt: If, cgs: &mut CodeGenStatus) {
            let then_label = cgs.next_label("then");
            let else_label = cgs.next_label("else");
            let end_label = cgs.next_label("end");

            // 条件の評価（TypedExprなのでtodo!()）
            // TODO: 条件式の評価
            let cond_result = gen_typed_expr(*if_stmt.cond, cgs); // todo!()で条件式を評価した結果

            // 条件による分岐
            if if_stmt.else_branch.is_some() {
                println!(
                    "  br i1 %{}, label %{}, label %{}",
                    cond_result, then_label, else_label
                );
            } else {
                println!(
                    "  br i1 %{}, label %{}, label %{}",
                    cond_result, then_label, end_label
                );
            }

            // then ブロック
            println!("{}:", then_label);
            stmt(*if_stmt.then_branch, cgs);
            println!("  br label %{}", end_label);

            // else ブロック（存在する場合）
            if let Some(else_branch) = if_stmt.else_branch {
                println!("{}:", else_label);
                stmt(*else_branch, cgs);
                println!("  br label %{}", end_label);
            }

            // 終了ラベル
            println!("{}:", end_label);
        }

        pub fn r#while(while_stmt: While, cgs: &mut CodeGenStatus) {
            let cond_label = cgs.next_label("while_cond");
            let body_label = cgs.next_label("while_body");
            let end_label = cgs.next_label("while_end");

            // ループラベルをプッシュ
            cgs.push_loop_labels(end_label.clone(), cond_label.clone());

            // 条件の評価へジャンプ
            println!("  br label %{}", cond_label);

            // 条件評価ラベル
            println!("{}:", cond_label);
            // TODO: 条件式の評価
            let cond_result = gen_typed_expr(*while_stmt.cond, cgs); // todo!()で条件式を評価した結果
            println!(
                "  br i1 %{}, label %{}, label %{}",
                cond_result, body_label, end_label
            );

            // 本体ラベル
            println!("{}:", body_label);
            stmt(*while_stmt.body, cgs);
            println!("  br label %{}", cond_label);

            // 終了ラベル
            println!("{}:", end_label);

            // ループラベルをポップ
            cgs.pop_loop_labels();
        }

        pub fn r#do_while(do_while_stmt: DoWhile, cgs: &mut CodeGenStatus) {
            let body_label = cgs.next_label("do_body");
            let cond_label = cgs.next_label("do_cond");
            let end_label = cgs.next_label("do_end");

            // ループラベルをプッシュ
            cgs.push_loop_labels(end_label.clone(), cond_label.clone());

            // 本体へジャンプ
            println!("  br label %{}", body_label);

            // 本体ラベル
            println!("{}:", body_label);
            stmt(*do_while_stmt.body, cgs);
            println!("  br label %{}", cond_label);

            // 条件評価ラベル
            println!("{}:", cond_label);
            // TODO: 条件式の評価
            let cond_result = gen_typed_expr(*do_while_stmt.cond, cgs); // todo!()で条件式を評価した結果
            println!(
                "  br i1 %{}, label %{}, label %{}",
                cond_result, body_label, end_label
            );

            // 終了ラベル
            println!("{}:", end_label);

            // ループラベルをポップ
            cgs.pop_loop_labels();
        }

        pub fn r#for(for_stmt: For, cgs: &mut CodeGenStatus) {
            let cond_label = cgs.next_label("for_cond");
            let body_label = cgs.next_label("for_body");
            let step_label = cgs.next_label("for_step");
            let end_label = cgs.next_label("for_end");

            // ループラベルをプッシュ（continueはstepラベルへ）
            cgs.push_loop_labels(end_label.clone(), step_label.clone());

            // 初期化
            if let Some(_init) = for_stmt.init {
                let _ = gen_typed_expr(*_init, cgs);
            }

            // 条件の評価へジャンプ
            println!("  br label %{}", cond_label);

            // 条件評価ラベル
            println!("{}:", cond_label);
            if let Some(_cond) = for_stmt.cond {
                // TODO: 条件式の評価
                let cond_result = gen_typed_expr(*_cond, cgs); // todo!()で条件式を評価した結果
                println!(
                    "  br i1 %{}, label %{}, label %{}",
                    cond_result, body_label, end_label
                );
            } else {
                // 条件なし（無限ループ）
                println!("  br label %{}", body_label);
            }

            // 本体ラベル
            println!("{}:", body_label);
            stmt(*for_stmt.body, cgs);
            println!("  br label %{}", step_label);

            // ステップラベル
            println!("{}:", step_label);
            if let Some(_step) = for_stmt.step {
                // TODO: ステップ式の評価
                todo!()
            }
            println!("  br label %{}", cond_label);

            // 終了ラベル
            println!("{}:", end_label);

            // ループラベルをポップ
            cgs.pop_loop_labels();
        }

        pub fn r#switch(switch_stmt: Switch, cgs: &mut CodeGenStatus) {
            let end_label = cgs.next_label("switch_end");
            let default_label = cgs.next_label("switch_default");

            // breakラベルをプッシュ（switchではcontinueは使用不可なので空文字列）
            cgs.break_labels.push(end_label.clone());

            // TODO: switch条件式の評価
            let cond_result = gen_typed_expr(*switch_stmt.cond, cgs); // todo!()で条件式を評価した結果

            // switchの開始
            print!("  switch i64 %{}, label %{} [", cond_result, default_label);

            let mut case_labels = Vec::new();
            let mut has_default = false;

            // ケースラベルの生成
            for (i, case) in switch_stmt.cases.iter().enumerate() {
                match case {
                    SwitchCase::Case(case_stmt) => {
                        let case_label = cgs.next_label(&format!("case_{}", i));
                        case_labels.push((case_label.clone(), case));
                        let case_value = case_stmt.const_expr.to_string(); // todo!()でcase値を評価
                        print!("\n    i64 {}, label %{}", case_value, case_label);
                    }
                    SwitchCase::Default(_) => {
                        has_default = true;
                    }
                }
            }
            println!("\n  ]");

            // 各caseの処理
            for i in 0..case_labels.len() {
                let (label, case) = &case_labels[i];
                if let SwitchCase::Case(case_stmt) = case {
                    println!("{}:", label);
                    for stmt in &case_stmt.stmts {
                        super::stmt(*stmt.clone(), cgs);
                    }
                    // break文が無い場合は次のcaseへfall through
                    if i < case_labels.len() - 1 {
                        println!("  br label %{}", case_labels[i + 1].0);
                    }
                }
            }

            // defaultケースの処理
            println!("{}:", default_label);
            if has_default {
                for case in &switch_stmt.cases {
                    if let SwitchCase::Default(default_case) = case {
                        for stmt in &default_case.stmts {
                            super::stmt(*stmt.clone(), cgs);
                        }
                        break;
                    }
                }
            }
            println!("  br label %{}", end_label);

            // 終了ラベル
            println!("{}:", end_label);

            // breakラベルをポップ
            cgs.break_labels.pop();
        }
    }
}

// gen_stmt.rsの最後に追加する関数生成機能
fn gen_function(function: FunctionDef, cgs: &mut CodeGenStatus) {
    let name = function.sig.ident.clone();
    let params = function.param_names.clone();
    let args: Vec<String> = params.iter().map(|_| cgs.name_gen.next()).collect();

    println!(
        "define i64 @{}({}) {{",
        name.get_name(),
        args.iter()
            .map(|x| format!("i64 noundef %{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // return用の変数とラベルを設定
    let return_ptr = cgs.name_gen.next();
    let return_label = "return_label".to_string();
    println!("%{} = alloca i64", return_ptr);

    cgs.return_value_ptr = Some(return_ptr.clone());
    cgs.return_label = Some(return_label.clone());

    // 引数の処理
    for (i, param_name) in params.iter().enumerate() {
        let ptr = cgs.name_gen.next();
        println!("%{} = alloca i64", ptr);
        println!("store i64 %{}, ptr %{}", args[i], ptr);
        cgs.variables.insert(param_name.clone(), ptr);
    }

    // 関数本体の処理
    for stmt in function.body.into_vec() {
        gen_stmt::stmt(*stmt, cgs);
    }

    // 常にreturn_labelにジャンプ（return文がない場合のため）
    println!("br label %{}", return_label);

    // return_labelとreturn処理
    println!("{}:", return_label);
    println!("%val = load i64, ptr %{}", return_ptr);
    println!("ret i64 %val");

    println!("}}");

    // 関数終了時にreturn関連の情報をクリア
    cgs.return_value_ptr = None;
    cgs.return_label = None;
    cgs.variables.clear();
}

fn gen_top_level(top_level: TopLevel, cgs: &mut CodeGenStatus) {
    match top_level {
        TopLevel::FunctionDef(function_def) => gen_function(function_def, cgs),
        TopLevel::FunctionProto(_) => todo!(), // 関数プロトタイプは無視
        TopLevel::Stmt(stmt) => gen_stmt::stmt(stmt, cgs),
    }
}

pub fn generate_program(program: Program, cgs: &mut CodeGenStatus) {
    for item in program.items {
        gen_top_level(item, cgs);
    }
}
