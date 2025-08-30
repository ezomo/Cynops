use super::{SemaExpr, TypedExpr};
use crate::ast::*;

impl Expr {
    /// ExprをTypedExprに変換する
    /// ここでは型チェックも行う
    pub fn to_typed_expr(self) -> TypedExpr {
        entry(self)
    }
}

pub fn entry(exper: Expr) -> TypedExpr {
    match exper {
        Expr::Assign(a) => assign(a),
        Expr::Binary(a) => binary(a),
        Expr::Call(a) => call(a),
        Expr::Cast(a) => cast(a),
        Expr::Char(a) => char_lit(a),
        Expr::Comma(a) => comma(a),
        Expr::Variable(a) => variable(a),
        Expr::MemberAccess(a) => memberaccess(a),
        Expr::NumFloat(a) => num_float(a),
        Expr::NumInt(a) => num_int(a),
        Expr::Postfix(a) => postfix(a),
        Expr::Sizeof(a) => sizeof(a),
        Expr::String(a) => string_lit(a),
        Expr::Subscript(a) => subscript(a),
        Expr::Ternary(a) => ternary(a),
        Expr::Unary(a) => unary(a),
    }
}

/// 代入可能性の厳密チェック（完全一致のみ許可）
fn check_assignment_compatibility(lhs_type: &Type, rhs_type: &Type) -> Result<(), String> {
    if lhs_type == rhs_type {
        Ok(())
    } else {
        Err(format!(
            "型が完全に一致しません: {:?} vs {:?}",
            lhs_type, rhs_type
        ))
    }
}

/// 関数パラメータの型チェック（配列構文を禁止）
fn check_function_parameter_type(param_type: &Type) -> Result<(), String> {
    match param_type {
        // 配列型のパラメータは禁止
        Type::Array(_) => {
            Err("関数パラメータでの配列構文は禁止されています。配列ポインタ(*param)[]または(*param)[N]を使用してください".to_string())
        },
        _ => Ok(()),
    }
}

fn assign(a: crate::ast::expr::Assign) -> TypedExpr {
    let lhs = entry(*a.lhs);
    let rhs = entry(*a.rhs);

    // 厳密な型チェック - 配列の暗黙変換を禁止
    if let Err(err) = check_assignment_compatibility(&lhs.r#type, &rhs.r#type) {
        panic!("代入エラー: {}", err);
    }

    TypedExpr::new(
        lhs.r#type.clone(),
        *SemaExpr::assign(a.op, Box::new(lhs), Box::new(rhs)),
    )
}

fn binary(a: crate::ast::expr::Binary) -> TypedExpr {
    let lhs = entry(*a.lhs);
    let rhs = entry(*a.rhs);

    // 二項演算の型推論（厳密版）
    let result_type = match (&lhs.r#type, &rhs.r#type, &a.op) {
        // 比較演算子は常にbool(int)を返す
        (_, _, BinaryOp::Comparison(_)) => Type::Int,

        // 論理演算子も int を返す
        (_, _, BinaryOp::Logical(_)) => Type::Int,

        // 同じ型同士の算術演算
        (t1, t2, BinaryOp::Arithmetic(_)) if t1 == t2 => t1.clone(),

        // ポインタ演算
        (
            Type::Pointer(inner),
            Type::Int,
            BinaryOp::Arithmetic(Arithmetic::Plus | Arithmetic::Minus),
        ) => Type::Pointer(inner.clone()),
        (Type::Int, Type::Pointer(inner), BinaryOp::Arithmetic(Arithmetic::Plus)) => {
            Type::Pointer(inner.clone())
        }

        // 配列の暗黙変換は禁止
        (Type::Array(_), _, _) | (_, Type::Array(_), _) => {
            panic!(
                "配列型を直接二項演算で使用することはできません。要素アクセスまたは明示的なアドレス取得を行ってください"
            );
        }

        // 異なる型の場合はエラー
        _ => panic!("二項演算での型不一致: {:?} vs {:?}", lhs.r#type, rhs.r#type),
    };

    TypedExpr::new(
        result_type,
        *SemaExpr::binary(a.op, Box::new(lhs), Box::new(rhs)),
    )
}

fn call(a: crate::ast::expr::Call) -> TypedExpr {
    let func = entry(*a.func);
    let args: Vec<Box<TypedExpr>> = a
        .args
        .into_iter()
        .map(|arg| Box::new(entry(*arg)))
        .collect();

    // 関数型チェックと引数型チェック
    let return_type = match &func.r#type {
        Type::Func(func_type) => {
            if func_type.params[0] == Type::Void {
                &func_type.return_type
            } else {
                for param in &func_type.params {
                    if let Err(err) = check_function_parameter_type(param) {
                        panic!("関数パラメータエラー: {}", err);
                    }
                }

                if (args.len() != func_type.params.len())
                    && (func_type.params.last().unwrap() != &Type::DotDotDot)
                {
                    panic!(
                        "引数の個数が一致しません: expected {}, got {},{:?}",
                        func_type.params.len(),
                        args.len(),
                        func_type.params
                    );
                }

                for (i, (arg, param_type)) in args.iter().zip(&func_type.params).enumerate() {
                    if param_type == &Type::DotDotDot {
                        break;
                    }
                    if let Err(err) = check_assignment_compatibility(param_type, &arg.r#type) {
                        panic!("第{}引数の型エラー: {}", i + 1, err);
                    }
                }
                &func_type.return_type
            }
        }
        _ => {
            panic!("関数型ではありません: {:?}", func.r#type);
        }
    };

    TypedExpr::new(*return_type.clone(), SemaExpr::call(func, args))
}

fn cast(a: crate::ast::expr::Cast) -> TypedExpr {
    let expr = entry(*a.expr);
    let target_type = *a.r#type;

    //互換性を決めていないのでまだ作れない TODO

    TypedExpr::new(target_type.clone(), *SemaExpr::cast(target_type, expr))
}

fn char_lit(c: char) -> TypedExpr {
    TypedExpr::new(Type::Char, SemaExpr::char_lit(c))
}

fn comma(a: crate::ast::expr::Comma) -> TypedExpr {
    let assigns: Vec<TypedExpr> = a.assigns.into_iter().map(|expr| entry(expr)).collect();

    // コンマ演算子は最後の式の型を返す
    let result_type = assigns
        .last()
        .map(|expr| expr.r#type.clone())
        .unwrap_or(Type::Void);

    TypedExpr::new(result_type, SemaExpr::comma(assigns))
}

fn variable(a: crate::ast::expr::Variable) -> TypedExpr {
    let var_type = *a.r#type;
    TypedExpr::new(var_type, SemaExpr::ident(a.ident))
}

fn memberaccess(a: crate::ast::expr::MemberAccess) -> TypedExpr {
    let base = entry(*a.base);

    // メンバーアクセスの型チェック
    let member_type = match (&base.r#type, &a.kind) {
        // 構造体.メンバー
        (Type::Struct(struct_def), MemberAccessOp::Dot) => {
            // MemberDeclから実際のメンバー型を取得
            struct_def
                .member
                .iter()
                .find(|member| member.ident.name == a.member.name)
                .map(|member| member.ty.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "構造体 {} にメンバー {} が見つかりません",
                        struct_def
                            .ident
                            .as_ref()
                            .map(|id| id.name.as_str())
                            .unwrap_or("Anonymous"),
                        a.member.name
                    )
                })
        }
        // 共用体.メンバー
        (Type::Union(union_def), MemberAccessOp::Dot) => {
            // MemberDeclから実際のメンバー型を取得
            union_def
                .member
                .iter()
                .find(|member| member.ident.name == a.member.name)
                .map(|member| member.ty.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "共用体 {} にメンバー {} が見つかりません",
                        union_def
                            .ident
                            .as_ref()
                            .map(|id| id.name.as_str())
                            .unwrap_or("Anonymous"),
                        a.member.name
                    )
                })
        }
        // ポインタ->メンバー
        (Type::Pointer(inner_type), MemberAccessOp::MinusGreater) => {
            match inner_type.as_ref() {
                Type::Struct(struct_def) => {
                    // MemberDeclから実際のメンバー型を取得
                    struct_def
                        .member
                        .iter()
                        .find(|member| member.ident.name == a.member.name)
                        .map(|member| member.ty.clone())
                        .unwrap_or_else(|| {
                            panic!(
                                "構造体 {} にメンバー {} が見つかりません",
                                struct_def
                                    .ident
                                    .as_ref()
                                    .map(|id| id.name.as_str())
                                    .unwrap_or("Anonymous"),
                                a.member.name
                            )
                        })
                }
                Type::Union(union_def) => {
                    // MemberDeclから実際のメンバー型を取得
                    union_def
                        .member
                        .iter()
                        .find(|member| member.ident.name == a.member.name)
                        .map(|member| member.ty.clone())
                        .unwrap_or_else(|| {
                            panic!(
                                "共用体 {} にメンバー {} が見つかりません",
                                union_def
                                    .ident
                                    .as_ref()
                                    .map(|id| id.name.as_str())
                                    .unwrap_or("Anonymous"),
                                a.member.name
                            )
                        })
                }
                _ => {
                    panic!(
                        "-> 演算子はポインタ型の構造体または共用体に対してのみ使用可能です: {:?}",
                        inner_type
                    );
                }
            }
        }
        // 配列のメンバーアクセスは基本的に禁止
        (Type::Array(_), _) => {
            panic!("配列型に対する直接のメンバーアクセスは禁止されています");
        }
        _ => {
            panic!(
                "不正なメンバーアクセス: {:?} に対して {:?}",
                base.r#type, a.kind
            );
        }
    };

    TypedExpr::new(member_type, SemaExpr::member_access(base, a.member, a.kind))
}

fn num_float(f: ordered_float::OrderedFloat<f64>) -> TypedExpr {
    TypedExpr::new(Type::Double, SemaExpr::num_float(f))
}

fn num_int(n: usize) -> TypedExpr {
    TypedExpr::new(Type::Int, SemaExpr::num_int(n))
}

fn postfix(a: crate::ast::expr::Postfix) -> TypedExpr {
    let expr = entry(*a.expr);

    // 配列に対する後置演算子は禁止
    match &expr.r#type {
        Type::Array(_) => {
            panic!("配列型に対する後置演算子は禁止されています");
        }
        _ => {}
    }

    let result_type = expr.r#type.clone();
    TypedExpr::new(result_type, SemaExpr::postfix(a.op, expr))
}

fn sizeof(a: crate::ast::expr::Sizeof) -> TypedExpr {
    let sema_sizeof = match a {
        crate::ast::expr::Sizeof::Type(ty) => crate::sema::Sizeof::Type(ty),
        crate::ast::expr::Sizeof::Expr(expr) => crate::sema::Sizeof::Expr(Box::new(entry(*expr))),
    };

    TypedExpr::new(Type::Int, *SemaExpr::sizeof(sema_sizeof))
}

fn string_lit(s: Vec<char>) -> TypedExpr {
    TypedExpr::new(
        Type::Array(Array {
            array_of: Box::new(Type::Char),
            length: s.len(),
        }), // 文字列リテラルはchar [n]型
        SemaExpr::string(s),
    )
}

fn subscript(a: crate::ast::expr::Subscript) -> TypedExpr {
    let name = entry(*a.name);
    let index = entry(*a.index);

    // インデックスは整数型である必要がある
    if !matches!(index.r#type, Type::Int) {
        panic!(
            "配列インデックスは整数型である必要があります: {:?}",
            index.r#type
        );
    }

    // 配列の添え字アクセス
    let element_type = match &name.r#type {
        Type::Array(arr) => *arr.array_of.clone(),
        Type::Pointer(elem_type) => {
            match elem_type.as_ref() {
                // 配列ポインタ(*arr)[N] の添え字アクセスはエラー
                // 正しくは (*arr)[index] のように間接参照してからアクセス
                Type::Array(_) => {
                    panic!(
                        "配列ポインタに対する添え字アクセスは (*ptr)[index] の形で間接参照を明示してください"
                    );
                }
                // 通常のポインタの添え字アクセス
                _ => *elem_type.clone(),
            }
        }
        _ => panic!(
            "添え字アクセスは配列またはポインタに対してのみ可能です: {:?}",
            name.r#type
        ),
    };

    TypedExpr::new(element_type, SemaExpr::subscript(name, index))
}

fn ternary(a: crate::ast::expr::Ternary) -> TypedExpr {
    let cond = entry(*a.cond);
    let then_branch = entry(*a.then_branch);
    let else_branch = entry(*a.else_branch);

    // 条件式の型チェック
    if !matches!(cond.r#type, Type::Int | Type::Pointer(_)) {
        panic!(
            "三項演算子の条件式は整数型またはポインタ型である必要があります: {:?}",
            cond.r#type
        );
    }

    // then節とelse節の型の厳密チェック
    if let Err(err) = check_assignment_compatibility(&then_branch.r#type, &else_branch.r#type) {
        // 逆方向もチェック
        if let Err(_) = check_assignment_compatibility(&else_branch.r#type, &then_branch.r#type) {
            panic!("三項演算子の分岐の型が一致しません: {}", err);
        }
    }

    let result_type = then_branch.r#type.clone();
    TypedExpr::new(
        result_type,
        *SemaExpr::ternary(Box::new(cond), then_branch, else_branch),
    )
}

fn unary(a: crate::ast::expr::Unary) -> TypedExpr {
    let expr = entry(*a.expr);

    // 単項演算子の型推論と配列チェック
    let result_type = match a.op {
        UnaryOp::Ampersand => {
            // アドレス演算子: 配列の場合は配列ポインタを返す
            match &expr.r#type {
                Type::Array(arr) => Type::Pointer(Box::new(Type::Array(arr.clone()))),
                _ => Type::Pointer(Box::new(expr.r#type.clone())),
            }
        }
        UnaryOp::Asterisk => {
            // 間接参照演算子
            match &expr.r#type {
                Type::Pointer(inner_type) => {
                    match inner_type.as_ref() {
                        // 配列ポインタの間接参照は配列型を返す
                        Type::Array(arr) => Type::Array(arr.clone()),
                        _ => *inner_type.clone(),
                    }
                }
                _ => panic!(
                    "間接参照はポインタ型に対してのみ可能です: {:?}",
                    expr.r#type
                ),
            }
        }
        UnaryOp::PlusPlus | UnaryOp::MinusMinus => {
            // インクリメント/デクリメント: 配列には適用不可
            match &expr.r#type {
                Type::Array(_) => {
                    panic!("配列型に対するインクリメント/デクリメント演算子は禁止されています");
                }
                _ => expr.r#type.clone(),
            }
        }
        UnaryOp::Minus => {
            // 単項-: 数値型のみ
            match &expr.r#type {
                Type::Int | Type::Double => expr.r#type.clone(),
                _ => panic!("単項-は数値型にのみ適用可能です: {:?}", expr.r#type),
            }
        }
        UnaryOp::Bang => {
            // 論理否定: 整数型またはポインタ型
            match &expr.r#type {
                Type::Int | Type::Pointer(_) => Type::Int,
                _ => panic!(
                    "論理否定は整数型またはポインタ型にのみ適用可能です: {:?}",
                    expr.r#type
                ),
            }
        }
        UnaryOp::Tilde => {
            // ビット反転: 整数型のみ
            match &expr.r#type {
                Type::Int => expr.r#type.clone(),
                _ => panic!("ビット反転は整数型にのみ適用可能です: {:?}", expr.r#type),
            }
        }
    };

    TypedExpr::new(result_type, *SemaExpr::unary(a.op, Box::new(expr)))
}
