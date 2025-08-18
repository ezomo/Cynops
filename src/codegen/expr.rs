use super::*;
use crate::ast::*;
use crate::sema::Subscript;
use crate::sema::*; // Explicitly import Subscript to disambiguate

pub fn gen_expr(typed_expr: TypedExpr, cgs: &mut CodeGenStatus) -> String {
    match typed_expr.expr {
        SemaExpr::Binary(binary) => match binary.op {
            BinaryOp::Arithmetic(ari) => {
                let lhs = gen_expr(*binary.lhs, cgs);
                let rhs = gen_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.value();

                println!("{} = {} i64 {}, {}", name, ari.to_llvmir(), lhs, rhs);
                name
            }
            BinaryOp::Comparison(com) => {
                let lhs = gen_expr(*binary.lhs, cgs);
                let rhs = gen_expr(*binary.rhs, cgs);
                let name = cgs.name_gen.value();

                println!("{} = {} i64 {}, {}", name, com.to_llvmir(), lhs, rhs);
                name
            }
            BinaryOp::Logical(Logical::AmpersandAmpersand) => {
                // 短絡評価: lhs && rhs
                let lhs = gen_expr(*binary.lhs, cgs);
                let lhs_bool = i64toi1(lhs, cgs);

                let true_label = cgs.name_gen.value();
                let false_label = cgs.name_gen.value();
                let end_label = cgs.name_gen.value();

                println!(
                    "br i1 {}, label %{}, label %{}",
                    lhs_bool, true_label, false_label
                );

                // true branch
                println!("{}:", true_label);
                let rhs = gen_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label);

                // false branch
                println!("{}:", false_label);
                println!("br label %{}", end_label);

                // end
                println!("{}:", end_label);
                let result = cgs.name_gen.value();
                println!(
                    "{} = phi i64 [{}, {}], [0, {}]",
                    result, rhs, true_label, false_label
                );
                result
            }
            BinaryOp::Logical(Logical::PipePipe) => {
                // 短絡評価: lhs || rhs
                let lhs = gen_expr(*binary.lhs, cgs);
                let lhs_bool = i64toi1(lhs.clone(), cgs);

                let false_label = cgs.name_gen.value();
                let true_label = cgs.name_gen.value();
                let end_label = cgs.name_gen.value();

                println!(
                    "br i1 {}, label %{}, label %{}",
                    lhs_bool, true_label, false_label
                );

                // false branch
                println!("{}:", false_label);
                let rhs = gen_expr(*binary.rhs, cgs);
                println!("br label %{}", end_label);

                // true branch
                println!("{}:", true_label);
                println!("br label %{}", end_label);

                // end
                println!("{}:", end_label);
                let result = cgs.name_gen.value();
                println!(
                    "{} = phi i64 [{}, {}], [{}, {}]",
                    result, lhs, true_label, rhs, false_label
                );
                result
            }
        },
        SemaExpr::Assign(assign) => match assign.op {
            AssignOp::Equal => {
                if let SemaExpr::Ident(ident) = &assign.lhs.r#expr {
                    let rhs = gen_expr(*assign.rhs, cgs);
                    let ptr = cgs.variables.get(ident).unwrap();
                    println!("store i64 {}, ptr {}", rhs, ptr);
                    rhs
                } else {
                    panic!("The left side is not variable!");
                }
            }
            _ => {
                // 複合代入演算子 (+=, -=, など)
                if let SemaExpr::Ident(ident) = &assign.lhs.r#expr {
                    let ptr = cgs.variables.get(ident).unwrap().clone();
                    let lhs_val = cgs.name_gen.value();

                    println!("{} = load i64, ptr {}", lhs_val, ptr);

                    let rhs = gen_expr(*assign.rhs, cgs);
                    let result = cgs.name_gen.value();

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

                    println!("{} = {} i64 {}, {}", result, op, lhs_val, rhs);
                    println!("store i64 {}, ptr {}", result, ptr);
                    result
                } else {
                    panic!("The left side is not variable!");
                }
            }
        },
        SemaExpr::NumInt(num) => {
            let name1 = cgs.name_gen.value();
            println!("{} = add i64 0, {}", name1, num);
            name1
        }
        SemaExpr::NumFloat(num) => {
            let name1 = cgs.name_gen.value();
            println!("{} = fadd double 0.0, {}", name1, num.0);
            name1
        }
        SemaExpr::Char(ch) => {
            let name1 = cgs.name_gen.value();
            println!("{} = add i8 0, {}", name1, ch as u8);
            name1
        }
        SemaExpr::String(s) => {
            let global_name = cgs.get_or_create_string_literal(&s);
            let name = cgs.name_gen.value();
            println!(
                "{} = getelementptr inbounds [{}x i8], ptr @{}, i64 0, i64 0",
                name,
                s.len() + 1,
                global_name
            );
            name
        }
        SemaExpr::Ident(ident) => match typed_expr.r#type {
            Type::Array(_) => cgs.variables[&ident].clone(),
            _ => {
                let tmp = cgs.name_gen.value();
                let ptr = cgs.variables.get(&ident).unwrap();
                println!("{} = load i64, ptr {}", tmp, ptr);
                tmp
            }
        },
        SemaExpr::Call(call) => {
            let name = cgs.name_gen.value();
            let args: Vec<String> = call
                .args
                .iter()
                .map(|arg| format!("i64 noundef {}", gen_expr(*arg.clone(), cgs)))
                .collect();

            let fn_name = match &call.func.r#expr {
                SemaExpr::Ident(idn) => idn.clone(),
                _ => panic!("Function call target is not an identifier"),
            };
            println!(
                "{} = call i64 @{}({})",
                name,
                fn_name.get_name(),
                args.join(", ")
            );
            name
        }
        SemaExpr::Unary(unary) => {
            match unary.op {
                UnaryOp::Minus => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.value();
                    println!("{} = sub i64 0, {}", name, operand);
                    name
                }
                UnaryOp::Bang => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.value();
                    println!("{} = icmp eq i64 {}, 0", name, operand);
                    i1toi64(name, cgs)
                }
                UnaryOp::Tilde => {
                    let operand = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.value();
                    println!("{} = xor i64 {}, -1", name, operand);
                    name
                }
                UnaryOp::PlusPlus => {
                    // 前置インクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.value();
                        println!("{} = load i64, ptr {}", old_val, ptr);
                        let new_val = cgs.name_gen.value();
                        println!("{} = add i64 {}, 1", new_val, old_val);
                        println!("store i64 {}, ptr {}", new_val, ptr);
                        new_val
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                UnaryOp::MinusMinus => {
                    // 前置デクリメント
                    if let SemaExpr::Ident(ident) = &unary.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.value();
                        println!("{} = load i64, ptr {}", old_val, ptr);
                        let new_val = cgs.name_gen.value();
                        println!("{} = sub i64 {}, 1", new_val, old_val);
                        println!("store i64 {}, ptr {}", new_val, ptr);
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
                    let ptr = gen_expr(*unary.expr, cgs);
                    let name = cgs.name_gen.value();
                    println!(
                        "{} = load {}, ptr {}",
                        name,
                        typed_expr
                            .r#type
                            .as_array()
                            .unwrap()
                            .array_of
                            .get_llvm_type(),
                        ptr
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
                        let old_val = cgs.name_gen.value();
                        println!("{} = load i64, ptr {}", old_val, ptr);
                        let new_val = cgs.name_gen.value();
                        println!("{} = add i64 {}, 1", new_val, old_val);
                        println!("store i64 {}, ptr {}", new_val, ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("++ can only be applied to variables");
                    }
                }
                PostfixOp::MinusMinus => {
                    // 後置デクリメント
                    if let SemaExpr::Ident(ident) = &postfix.expr.r#expr {
                        let ptr = cgs.variables.get(ident).unwrap().clone();
                        let old_val = cgs.name_gen.value();
                        println!("{} = load i64, ptr {}", old_val, ptr);
                        let new_val = cgs.name_gen.value();
                        println!("{} = sub i64 {}, 1", new_val, old_val);
                        println!("store i64 {}, ptr {}", new_val, ptr);
                        old_val // 後置なので古い値を返す
                    } else {
                        panic!("-- can only be applied to variables");
                    }
                }
            }
        }
        SemaExpr::Ternary(ternary) => {
            let cond = gen_expr(*ternary.cond, cgs);
            let cond_bool = i64toi1(cond, cgs);

            let true_label = cgs.name_gen.value();
            let false_label = cgs.name_gen.value();
            let end_label = cgs.name_gen.value();

            println!(
                "br i1 {}, label %{}, label %{}",
                cond_bool, true_label, false_label
            );

            // true branch
            println!("{}:", true_label);
            let true_val = gen_expr(*ternary.then_branch, cgs);
            println!("br label %{}", end_label);

            // false branch
            println!("{}:", false_label);
            let false_val = gen_expr(*ternary.else_branch, cgs);
            println!("br label %{}", end_label);

            // end
            println!("{}:", end_label);
            let result = cgs.name_gen.value();
            println!(
                "{} = phi i64 [{}, {}], [{}, {}]",
                result, true_val, true_label, false_val, false_label
            );
            result
        }
        SemaExpr::Subscript(subscript) => {
            // TODO
            fn array_access(subscript: Subscript, cgs: &mut CodeGenStatus) -> String {
                let inside_type = subscript.subject.r#type.get_llvm_type();
                let arr_ptr = gen_expr(*subscript.subject, cgs);
                let index = gen_expr(*subscript.index, cgs);

                let name = cgs.name_gen.value();
                println!(
                    "{} = getelementptr inbounds {}, ptr {}, i64 0 ,i64 {}",
                    name, inside_type, arr_ptr, index
                );

                name
            }

            let result = array_access(subscript, cgs);
            if !matches!(typed_expr.r#type, Type::Array(_)) {
                let name = cgs.name_gen.value();
                println!(
                    "{} = load {}, ptr {}",
                    name,
                    typed_expr.r#type.get_llvm_type(),
                    result
                );
                name
            } else {
                result
            }
        }
        SemaExpr::MemberAccess(member_access) => {
            // 構造体メンバアクセス
            let base = gen_expr(*member_access.base, cgs);
            match member_access.kind {
                MemberAccessOp::Dot => {
                    // obj.member
                    let name = cgs.name_gen.value();
                    println!(
                        "{} = getelementptr inbounds struct, ptr {}, i64 0, i64 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.value();
                    println!("{} = load i64, ptr {}", result, name);
                    result
                }
                MemberAccessOp::MinusGreater => {
                    // ptr->member
                    let name = cgs.name_gen.value();
                    println!(
                        "{} = getelementptr inbounds struct, ptr {}, i64 0, i64 {}",
                        name, base, 0
                    ); // 簡略化のため0番目として扱う
                    let result = cgs.name_gen.value();
                    println!("{} = load i64, ptr {}", result, name);
                    result
                }
            }
        }
        SemaExpr::Sizeof(_sizeof) => {
            // sizeof演算子 - 簡略化のため4（intのサイズ）を返す
            let name = cgs.name_gen.value();
            println!("{} = add i64 0, 4", name);
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
            let mut last_val = "".to_string();
            for assign in comma.assigns {
                last_val = gen_expr(assign, cgs);
            }
            last_val
        }
    }
}
