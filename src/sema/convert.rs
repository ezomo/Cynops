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

fn assign(a: Assign) -> TypedExpr {
    let lhs = entry(*a.lhs);
    let rhs = entry(*a.rhs);

    // 型チェック - 代入では左辺と右辺の型が一致している必要がある
    if lhs.r#type == rhs.r#type {
        TypedExpr::new(
            lhs.r#type.clone(),
            *SemaExpr::assign(a.op, Box::new(lhs), Box::new(rhs)),
        )
    } else {
        panic!(
            "Type mismatch in assignment: {:?} vs {:?}",
            lhs.r#type, rhs.r#type
        )
    }
}

fn binary(a: Binary) -> TypedExpr {
    let lhs = entry(*a.lhs);
    let rhs = entry(*a.rhs);

    // 二項演算の型推論（簡易版）
    let result_type = match (&lhs.r#type, &rhs.r#type) {
        // 同じ型同士の演算は同じ型を返す
        (t1, t2) if t1 == t2 => t1.clone(),
        // 異なる型の場合はエラー（実際のCコンパイラではより複雑な型変換ルールがある）
        _ => panic!(
            "Type mismatch in binary operation: {:?} vs {:?}",
            lhs.r#type, rhs.r#type
        ),
    };

    TypedExpr::new(
        result_type,
        *SemaExpr::binary(a.op, Box::new(lhs), Box::new(rhs)),
    )
}

fn call(a: Call) -> TypedExpr {
    let func = entry(*a.func);
    let args: Vec<Box<TypedExpr>> = a
        .args
        .into_iter()
        .map(|arg| Box::new(entry(*arg)))
        .collect();

    // 関数呼び出しの戻り値型を決定（実際にはシンボルテーブルから取得）
    let return_type = match &func.r#type {
        Type::Func(func_type) => match &func_type.return_type {
            Some(ret_type) => *ret_type.clone(),
            None => Type::Void,
        },
        _ => {
            // 関数型でない場合はエラー（実際にはもっと詳細なエラー処理が必要）
            Type::Void
        }
    };

    TypedExpr::new(return_type, SemaExpr::call(func, args))
}

fn cast(a: Cast) -> TypedExpr {
    let expr = entry(*a.expr);
    let target_type = *a.r#type;

    TypedExpr::new(target_type.clone(), *SemaExpr::cast(target_type, expr))
}

fn char_lit(c: char) -> TypedExpr {
    TypedExpr::new(Type::Char, SemaExpr::char_lit(c))
}

fn comma(a: Comma) -> TypedExpr {
    let assigns: Vec<TypedExpr> = a.assigns.into_iter().map(|expr| entry(expr)).collect();

    // コンマ演算子は最後の式の型を返す
    let result_type = assigns
        .last()
        .map(|expr| expr.r#type.clone())
        .unwrap_or(Type::Void);

    TypedExpr::new(result_type, SemaExpr::comma(assigns))
}

fn variable(a: Variable) -> TypedExpr {
    // 識別子の型はシンボルテーブルから取得する必要がある
    // ここでは仮にint型とする
    let var_type = a.r#type; // 実際の実装ではシンボルテーブルから取得

    TypedExpr::new(*var_type, SemaExpr::ident(a.ident))
}

fn memberaccess(a: MemberAccess) -> TypedExpr {
    let base = entry(*a.base);

    // メンバーアクセスの型は構造体定義から取得する必要がある
    // ここでは仮にint型とする
    let member_type = Type::Int; // 実際の実装では構造体定義から取得

    TypedExpr::new(member_type, SemaExpr::member_access(base, a.member, a.kind))
}

fn num_float(f: ordered_float::OrderedFloat<f64>) -> TypedExpr {
    TypedExpr::new(
        Type::Double, // またはType::Float
        SemaExpr::num_float(f),
    )
}

fn num_int(n: usize) -> TypedExpr {
    TypedExpr::new(Type::Int, SemaExpr::num_int(n))
}

fn postfix(a: Postfix) -> TypedExpr {
    let expr = entry(*a.expr);
    let result_type = expr.r#type.clone();

    TypedExpr::new(result_type, SemaExpr::postfix(a.op, expr))
}

fn sizeof(a: Sizeof) -> TypedExpr {
    // expr::Sizeof を sema::Sizeof に変換
    let sema_sizeof = match a {
        Sizeof::Type(ty) => crate::sema::Sizeof::Type(ty),
        Sizeof::Expr(expr) => crate::sema::Sizeof::Expr(Box::new(entry(*expr))),
    };

    TypedExpr::new(
        Type::Int, // sizeofはsize_t型だが、ここではint型とする
        *SemaExpr::sizeof(sema_sizeof),
    )
}

fn string_lit(s: String) -> TypedExpr {
    TypedExpr::new(
        Type::Pointer(Box::new(Type::Char)), // 文字列リテラルはchar*型
        SemaExpr::string(s),
    )
}

fn subscript(a: Subscript) -> TypedExpr {
    let name = entry(*a.name);
    let index = entry(*a.index);

    // 配列の添え字アクセスは配列の要素型を返す
    let element_type = match &name.r#type {
        Type::Array(arr) => *arr.array_of.clone(),
        Type::Pointer(elem_type) => *elem_type.clone(),
        _ => panic!("Subscript operation on non-array type: {:?}", name.r#type),
    };

    TypedExpr::new(element_type, SemaExpr::subscript(name, index))
}

fn ternary(a: Ternary) -> TypedExpr {
    let cond = entry(*a.cond);
    let then_branch = entry(*a.then_branch);
    let else_branch = entry(*a.else_branch);

    // 三項演算子はthen節とelse節の型が一致している必要がある
    if then_branch.r#type == else_branch.r#type {
        let result_type = then_branch.r#type.clone();
        TypedExpr::new(
            result_type,
            *SemaExpr::ternary(Box::new(cond), then_branch, else_branch),
        )
    } else {
        panic!(
            "Type mismatch in ternary operator: {:?} vs {:?}",
            then_branch.r#type, else_branch.r#type
        );
    }
}

fn unary(a: Unary) -> TypedExpr {
    let expr = entry(*a.expr);

    // 単項演算子の型推論
    let result_type = match a.op {
        UnaryOp::Ampersand => {
            // アドレス演算子はポインタ型を返す
            Type::Pointer(Box::new(expr.r#type.clone()))
        }
        UnaryOp::Asterisk => {
            // 間接参照演算子はポインタの指す型を返す
            match &expr.r#type {
                Type::Pointer(inner_type) => *inner_type.clone(),
                _ => panic!(
                    "Dereference operation on non-pointer type: {:?}",
                    expr.r#type
                ),
            }
        }
        UnaryOp::PlusPlus | UnaryOp::MinusMinus => {
            // インクリメント/デクリメントは元の型を返す
            expr.r#type.clone()
        }
        _ => {
            // その他の単項演算子（-, !, ~）は元の型を返す
            expr.r#type.clone()
        }
    };

    TypedExpr::new(result_type, *SemaExpr::unary(a.op, Box::new(expr)))
}
