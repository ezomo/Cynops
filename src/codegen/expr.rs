use ordered_float::OrderedFloat;

use super::*;
use crate::ast::*;
use crate::sema::Subscript;
use crate::sema::*; // Explicitly import Subscript to disambiguate

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) -> LLVMValue {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => match binary.op {
            BinaryOp::Arithmetic(ari) => {
                let lhs = load(&typed_expr.r#type, gen_expr(*binary.lhs, cgs), cgs);
                let rhs = load(&typed_expr.r#type, gen_expr(*binary.rhs, cgs), cgs);

                let name = cgs.name_gen.register();

                println!(
                    "{} = {} {} {}, {}",
                    name.to_string(),
                    ari.to_llvmir(),
                    typed_expr.r#type.to_llvm_format(),
                    lhs.to_string(),
                    rhs.to_string()
                );
                name
            }
            BinaryOp::Comparison(com) => {
                let lhs = new_load(gen_expr, *binary.lhs, cgs);
                let rhs = new_load(gen_expr, *binary.rhs, cgs);
                let name = cgs.name_gen.register();

                println!(
                    "{} = {} {} {}, {}",
                    name.to_string(),
                    com.to_llvmir(),
                    typed_expr.r#type.to_llvm_format(),
                    lhs.to_string(),
                    rhs.to_string()
                );
                name.i1toi64(cgs)
            }
            BinaryOp::Logical(Logical::AmpersandAmpersand) => {
                // 短絡評価: lhs && rhs
                let lhs_bool = gen_expr(*binary.lhs, cgs).i64toi1(cgs);
                let true_label = cgs.name_gen.label();
                let false_label = cgs.name_gen.label();
                let end_label = cgs.name_gen.label();

                println!(
                    "br i1 {}, label %{}, label %{}",
                    lhs_bool.to_string(),
                    true_label.to_string(),
                    false_label.to_string()
                );

                // true branch
                println!("{}:", true_label.to_string());
                let rhs = gen_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label.to_string());

                // false branch
                println!("{}:", false_label.to_string());
                println!("br label %{}", end_label.to_string());

                // end
                println!("{}:", end_label.to_string());
                let result = cgs.name_gen.register();
                println!(
                    "{} = phi i64 [{}, %{}], [0, %{}]",
                    result.to_string(),
                    rhs.to_string(),
                    true_label.to_string(),
                    false_label.to_string()
                );
                result
            }
            BinaryOp::Logical(Logical::PipePipe) => {
                // 短絡評価: lhs || rhs
                let lhs = gen_expr(*binary.lhs, cgs);
                let lhs_bool = lhs.clone().i64toi1(cgs);

                let false_label = cgs.name_gen.register();
                let true_label = cgs.name_gen.register();
                let end_label = cgs.name_gen.register();

                println!(
                    "br i1 {}, label %{}, label %{}",
                    lhs_bool.to_string(),
                    true_label.to_string(),
                    false_label.to_string()
                );

                // false branch
                println!("{}:", false_label.to_string());
                let rhs = gen_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label.to_string());

                // true branch
                println!("{}:", true_label.to_string());
                println!("br label %{}", end_label.to_string());

                // end
                println!("{}:", end_label.to_string());
                let result = cgs.name_gen.register();
                println!(
                    "{} = phi i64 [{}, {}], [{}, {}]",
                    result.to_string(),
                    lhs.to_string(),
                    true_label.to_string(),
                    rhs.to_string(),
                    false_label.to_string()
                );
                result
            }
        },
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                let rhs = load(&typed_expr.r#type, gen_expr(*assign.rhs.clone(), cgs), cgs);
                let ptr = gen_expr(*assign.lhs, cgs);
                println!(
                    "store {} {}, {}* {}",
                    typed_expr.r#type.to_llvm_format(),
                    rhs.to_string(),
                    typed_expr.r#type.to_llvm_format(),
                    ptr.to_string()
                );
                rhs
            }
            _ => {
                // 複合代入演算子 (+=, -=, など)
                if let SemaExpr::Ident(ident) = &assign.lhs.r#expr {
                    let ptr = cgs.variables.get(ident).unwrap().clone();
                    let lhs_val = cgs.name_gen.register();

                    println!("{} = load i64, ptr {}", lhs_val.to_string(), ptr);

                    let rhs = gen_expr(*assign.rhs, cgs);
                    let result = cgs.name_gen.register();

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

                    println!(
                        "{} = {} {} {}, {}",
                        result.to_string(),
                        op,
                        typed_expr.r#type.to_llvm_format(),
                        lhs_val.to_string(),
                        rhs.to_string()
                    );
                    println!(
                        "store {} {}, ptr {}",
                        typed_expr.r#type.to_llvm_format(),
                        result.to_string(),
                        ptr
                    );
                    result
                } else {
                    panic!("The left side is not variable!");
                }
            }
        },
        SemaExpr::NumInt(num) => LLVMValue::new(num, LLVMType::Const),
        SemaExpr::NumFloat(num) => {
            fn format_float(x: OrderedFloat<f64>) -> String {
                let s = x.0.to_string();
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    // すでに小数点 or 指数表記が含まれていればそのまま
                    s
                } else {
                    // 整数だった場合だけ .0 を追加
                    format!("{}.0", s)
                }
            }
            LLVMValue::new(format_float(num), LLVMType::Const)
        }
        SemaExpr::Char(ch) => {
            let name1 = cgs.name_gen.register();
            println!("{} = add i8 0, {}", name1.to_string(), ch as u8);
            name1
        }
        SemaExpr::String(s) => {
            let var_name = cgs.name_gen.variable();
            println!(
                "{} = alloca {}",
                var_name.to_string(),
                typed_expr.r#type.to_llvm_format()
            );
            let arr = typed_expr.r#type.as_array().unwrap();
            for i in 0..arr.length {
                let element_ptr = cgs.name_gen.register();
                let array_type = format!("[{} x {}]", arr.length, &arr.array_of.to_llvm_format());
                println!(
                    "  {} = getelementptr inbounds {}, {}* {}, i64 0, i64 {}",
                    element_ptr.to_string(),
                    array_type,
                    array_type,
                    var_name.to_string(),
                    i
                );

                println!(
                    "  store {} {}, {}* {}",
                    arr.array_of.to_llvm_format(),
                    s[i] as u8,
                    arr.array_of.to_llvm_format(),
                    element_ptr.to_string()
                );
            }

            var_name
        }
        SemaExpr::Ident(ident) => match typed_expr.r#type {
            Type::Array(_) => LLVMValue::new(cgs.variables[&ident].clone(), LLVMType::Variable),
            Type::Pointer(_) => LLVMValue::new(cgs.variables[&ident].clone(), LLVMType::Variable),
            _ => LLVMValue::new(
                cgs.variables.get(&ident).unwrap_or(&ident.get_fnc_name()),
                LLVMType::Variable,
            ),
        },
        SemaExpr::Call(call) => {
            let args: Vec<String> = call
                .args
                .iter()
                .map(|arg| {
                    format!(
                        "{} {}",
                        arg.r#type.to_llvm_format(),
                        load(&arg.r#type, gen_expr(*arg.clone(), cgs), cgs).to_string()
                    )
                })
                .collect::<Vec<String>>();
            // TODO
            let fn_name = gen_expr(*call.func.clone(), cgs);

            let return_type = &call.func.r#type.as_func().unwrap().return_type;
            let name = match **return_type {
                Type::Void => {
                    let name = cgs.name_gen.void();
                    println!(
                        "call {} {}({})",
                        call.func.r#type.to_llvm_format(),
                        fn_name.to_string(),
                        args.join(", ")
                    );
                    name
                }
                _ => {
                    let name = cgs.name_gen.register();
                    println!(
                        "{} = call {} {}({})",
                        name.to_string(),
                        call.func.r#type.to_llvm_format(),
                        fn_name.to_string(),
                        args.join(", ")
                    );
                    name
                }
            };

            wrap(&return_type, name, cgs)
        }
        SemaExpr::Unary(unary) => {
            match unary.op {
                UnaryOp::Minus => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.register();
                    println!("{} = sub i64 0, {}", name.to_string(), operand.to_string());
                    name
                }
                UnaryOp::Bang => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.register();
                    println!(
                        "{} = icmp eq i64 {}, 0",
                        name.to_string(),
                        operand.to_string()
                    );
                    name.i1toi64(cgs)
                }
                UnaryOp::Tilde => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.register();
                    println!("{} = xor i64 {}, -1", name.to_string(), operand.to_string());
                    name
                }
                UnaryOp::PlusPlus => {
                    // 前置インクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.register();
                        println!("{} = load i64, ptr {}", old_val.to_string(), ptr);
                        let new_val = cgs.name_gen.register();
                        println!(
                            "{} = add i64 {}, 1",
                            new_val.to_string(),
                            old_val.to_string()
                        );
                        println!("store i64 {}, ptr {}", new_val.to_string(), ptr);
                        new_val
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                UnaryOp::MinusMinus => {
                    // 前置デクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.register();
                        println!("{} = load i64, ptr {}", old_val.to_string(), ptr);
                        let new_val = cgs.name_gen.register();
                        println!(
                            "{} = sub i64 {}, 1",
                            new_val.to_string(),
                            old_val.to_string()
                        );
                        println!("store i64 {}, ptr {}", new_val.to_string(), ptr);
                        new_val
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
                UnaryOp::Ampersand => {
                    let ty = unary.expr.r#type.clone();
                    let val = wrap(&ty, gen_expr(*unary.expr, cgs), cgs);

                    // println!("{:?}", val);
                    let name = cgs.name_gen.variable();
                    println!("{} = alloca {}*", name.to_string(), ty.to_llvm_format());

                    println!(
                        "store {}* {}, {}** {}",
                        ty.to_llvm_format(),
                        val.to_string(),
                        ty.to_llvm_format(),
                        name.to_string()
                    );
                    name
                }
                UnaryOp::Asterisk => {
                    // 間接参照演算子 *x
                    let ptr_type = unary.expr.r#type.to_llvm_format();
                    let ptr = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.variable();
                    println!(
                        "{} = load {}*, {}* {}",
                        name.to_string(),
                        typed_expr.r#type.to_llvm_format(),
                        ptr_type,
                        ptr.to_string()
                    );
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
                        let old_val = cgs.name_gen.register();
                        println!("{} = load i64, ptr {}", old_val.to_string(), ptr);
                        let new_val = cgs.name_gen.register();
                        println!(
                            "{} = add i64 {}, 1",
                            new_val.to_string(),
                            old_val.to_string()
                        );
                        println!("store i64 {}, ptr {}", new_val.to_string(), ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                PostfixOp::MinusMinus => {
                    // 後置デクリメント
                    if let SemaExpr::Ident(ident) = &postfix.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.register();
                        println!("{} = load i64, ptr {}", old_val.to_string(), ptr);
                        let new_val = cgs.name_gen.register();
                        println!(
                            "{} = sub i64 {}, 1",
                            new_val.to_string(),
                            old_val.to_string()
                        );
                        println!("store i64 {}, ptr {}", new_val.to_string(), ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
            }
        }
        SemaExpr::Ternary(ternary) => {
            let cond = new_load(gen_expr, *ternary.cond, cgs);
            let cond_bool = cond.i64toi1(cgs);

            let true_label = cgs.name_gen.label();
            let false_label = cgs.name_gen.label();
            let end_label = cgs.name_gen.label();

            println!(
                "br i1 {}, label %{}, label %{}",
                cond_bool.to_string(),
                true_label.to_string(),
                false_label.to_string()
            );

            // true branch
            println!("{}:", true_label.to_string());
            let true_val = gen_expr(*ternary.then_branch, cgs);
            println!("br label %{}", end_label.to_string());

            // false branch
            println!("{}:", false_label.to_string());
            let false_val = gen_expr(*ternary.else_branch, cgs);
            println!("br label %{}", end_label.to_string());

            // end
            println!("{}:", end_label.to_string());
            let result = cgs.name_gen.register();
            println!(
                "{} = phi i64 [{}, %{}], [{}, %{}]",
                result.to_string(),
                true_val.to_string(),
                true_label.to_string(),
                false_val.to_string(),
                false_label.to_string()
            );
            result
        }
        SemaExpr::Subscript(subscript) => {
            fn array_access(subscript: Subscript, cgs: &mut CodeGenStatus) -> LLVMValue {
                let inside_type = subscript.subject.r#type.to_llvm_format();
                let arr_ptr = gen_expr(*subscript.subject, cgs);
                let index = new_load(gen_expr, *subscript.index, cgs);

                let name = cgs.name_gen.variable();
                println!(
                    "{} = getelementptr inbounds {}, {}* {}, i64 0 ,i64 {}",
                    name.to_string(),
                    inside_type,
                    inside_type,
                    arr_ptr.to_string(),
                    index.to_string()
                );

                name
            }

            array_access(subscript, cgs)
        }
        SemaExpr::MemberAccess(member_access) => {
            // 構造体メンバアクセス
            let base = gen_expr(*member_access.base, cgs);
            match member_access.kind {
                MemberAccessOp::Dot => {
                    // obj.member
                    let name = cgs.name_gen.register();
                    println!(
                        "{} = getelementptr inbounds struct, ptr {}, i64 0, i64 {}",
                        name.to_string(),
                        base.to_string(),
                        0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.register();
                    println!(
                        "{} = load i64, ptr {}",
                        result.to_string(),
                        name.to_string()
                    );
                    result
                }
                MemberAccessOp::MinusGreater => {
                    // ptr->member
                    let name = cgs.name_gen.register();
                    println!(
                        "{} = getelementptr inbounds struct, ptr {}, i64 0, i64 {}",
                        name.to_string(),
                        base.to_string(),
                        0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.register();
                    println!(
                        "{} = load i64, ptr {}",
                        result.to_string(),
                        name.to_string()
                    );
                    result
                }
            }
        }
        SemaExpr::Sizeof(_sizeof) => {
            // sizeof演算子 - 簡略化のため4（intのサイズ）を返す
            let name = cgs.name_gen.register();
            println!("{} = add i64 0, 4", name.to_string());
            name
        }
        SemaExpr::Cast(cast) => {
            // キャスト演算子 (type)expr
            let expr_val = gen_expr(*cast.expr, cgs);
            // 簡略化のため、実際の型変換は行わずそのまま返す
            expr_val
        }
        SemaExpr::Comma(comma) => {
            // カンマ演算子 - 最後の式の値を返す
            let mut last_val = gen_expr(comma.assigns[0].clone(), cgs);
            for i in 1..comma.assigns.len() {
                last_val = gen_expr(comma.assigns[i].clone(), cgs);
            }
            last_val
        }
    }
}
