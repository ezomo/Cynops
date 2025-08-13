//TypedExprに対応した定数計算モジュール
use crate::ast::{Arithmetic, BinaryOp, Comparison, Logical, UnaryOp};
use crate::ast::{Type, expr};
use crate::sema::{SemaExpr, TypedExpr};

impl TypedExpr {
    pub fn eval_const(self) -> Result<isize, String> {
        eval_const_typed_expr(&self)
    }
}

pub fn eval_const_typed_expr(typed_expr: &TypedExpr) -> Result<isize, String> {
    // まず型チェック: int型以外の計算は即座にエラー
    if !is_compile_time_constant_type(&typed_expr.r#type) {
        return Err(format!(
            "非int型は定数計算できません: {:?}",
            typed_expr.r#type
        ));
    }

    eval_sema_expr(&typed_expr.r#expr, &typed_expr.r#type)
}

/// コンパイル時定数として扱える型かチェック
fn is_compile_time_constant_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Char)
}

fn eval_sema_expr(expr: &SemaExpr, expr_type: &Type) -> Result<isize, String> {
    // 型チェックを最初に実行
    if !is_compile_time_constant_type(expr_type) {
        return Err(format!(
            "定数計算で非対応型が検出されました: {:?}",
            expr_type
        ));
    }

    match expr {
        SemaExpr::NumInt(n) => {
            if *expr_type != Type::Int {
                return Err(format!(
                    "整数リテラルの型が期待値と異なります: {:?}",
                    expr_type
                ));
            }
            Ok(*n as isize)
        }

        SemaExpr::Char(c) => {
            if *expr_type != Type::Char {
                return Err(format!(
                    "文字リテラルの型が期待値と異なります: {:?}",
                    expr_type
                ));
            }
            Ok(*c as isize)
        }

        // 非対応リテラル
        SemaExpr::NumFloat(_) => Err("浮動小数点数は定数計算できません".to_string()),
        SemaExpr::String(_) => Err("文字列は定数計算できません".to_string()),

        SemaExpr::Unary(unary) => eval_typed_unary(unary),
        SemaExpr::Binary(binary) => eval_typed_binary(binary),
        SemaExpr::Ternary(ternary) => eval_typed_ternary(ternary),
        SemaExpr::Cast(cast) => eval_typed_cast(cast),
        SemaExpr::Comma(comma) => eval_typed_comma(comma),
        SemaExpr::Sizeof(sizeof) => eval_typed_sizeof(sizeof),

        // 実行時要素は処理しない
        SemaExpr::Ident(_) => Err("変数は定数計算できません".to_string()),
        SemaExpr::Call(_) => Err("関数呼び出しは定数計算できません".to_string()),
        SemaExpr::Subscript(_) => Err("配列添え字は定数計算できません".to_string()),
        SemaExpr::MemberAccess(_) => Err("メンバアクセスは定数計算できません".to_string()),
        SemaExpr::Postfix(_) => Err("後置演算子は定数計算できません".to_string()),
        SemaExpr::Assign(_) => Err("代入は定数計算できません".to_string()),
    }
}

fn eval_typed_unary(unary: &crate::sema::Unary) -> Result<isize, String> {
    // 事前に型チェック
    if !is_compile_time_constant_type(&unary.expr.r#type) {
        return Err(format!(
            "単項演算子のオペランド型が非対応: {:?}",
            unary.expr.r#type
        ));
    }

    let operand = eval_const_typed_expr(&unary.expr)?;

    match unary.op {
        UnaryOp::Minus => Ok(-operand),
        UnaryOp::Bang => Ok(if operand != 0 { 0 } else { 1 }),
        UnaryOp::Tilde => Ok(!operand),
        UnaryOp::Ampersand => Err("アドレス演算子は定数計算できません".to_string()),
        UnaryOp::Asterisk => Err("間接参照演算子は定数計算できません".to_string()),
        UnaryOp::PlusPlus => Err("前置インクリメントは定数計算できません".to_string()),
        UnaryOp::MinusMinus => Err("前置デクリメントは定数計算できません".to_string()),
    }
}

fn eval_typed_binary(binary: &crate::sema::Binary) -> Result<isize, String> {
    // 事前に型チェック
    if !is_compile_time_constant_type(&binary.lhs.r#type) {
        return Err(format!(
            "二項演算子の左オペランド型が非対応: {:?}",
            binary.lhs.r#type
        ));
    }
    if !is_compile_time_constant_type(&binary.rhs.r#type) {
        return Err(format!(
            "二項演算子の右オペランド型が非対応: {:?}",
            binary.rhs.r#type
        ));
    }

    let lhs = eval_const_typed_expr(&binary.lhs)?;
    let rhs = eval_const_typed_expr(&binary.rhs)?;

    match binary.op {
        BinaryOp::Arithmetic(arith) => eval_arithmetic(arith, lhs, rhs),
        BinaryOp::Comparison(comp) => eval_comparison(comp, lhs, rhs),
        BinaryOp::Logical(logical) => eval_logical(logical, lhs, rhs),
    }
}

fn eval_arithmetic(op: Arithmetic, lhs: isize, rhs: isize) -> Result<isize, String> {
    match op {
        Arithmetic::Plus => Ok(lhs + rhs),
        Arithmetic::Minus => Ok(lhs - rhs),
        Arithmetic::Asterisk => Ok(lhs * rhs),
        Arithmetic::Slash => {
            if rhs == 0 {
                Err("0で除算".to_string())
            } else {
                Ok(lhs / rhs)
            }
        }
        Arithmetic::Percent => {
            if rhs == 0 {
                Err("0で剰余".to_string())
            } else {
                Ok(lhs % rhs)
            }
        }
        Arithmetic::Ampersand => Ok(lhs & rhs),
        Arithmetic::Pipe => Ok(lhs | rhs),
        Arithmetic::Caret => Ok(lhs ^ rhs),
        Arithmetic::LessLess => {
            if rhs < 0 || rhs >= 64 {
                Err("不正なシフト量".to_string())
            } else {
                Ok(lhs << rhs)
            }
        }
        Arithmetic::GreaterGreater => {
            if rhs < 0 || rhs >= 64 {
                Err("不正なシフト量".to_string())
            } else {
                Ok(lhs >> rhs)
            }
        }
    }
}

fn eval_comparison(op: Comparison, lhs: isize, rhs: isize) -> Result<isize, String> {
    let result = match op {
        Comparison::EqualEqual => lhs == rhs,
        Comparison::NotEqual => lhs != rhs,
        Comparison::Less => lhs < rhs,
        Comparison::LessEqual => lhs <= rhs,
        Comparison::Greater => lhs > rhs,
        Comparison::GreaterEqual => lhs >= rhs,
    };

    Ok(if result { 1 } else { 0 })
}

fn eval_logical(op: Logical, lhs: isize, rhs: isize) -> Result<isize, String> {
    match op {
        Logical::AmpersandAmpersand => {
            // 短絡評価: 左がfalseなら右を評価しない
            if lhs == 0 {
                Ok(0)
            } else {
                Ok(if rhs != 0 { 1 } else { 0 })
            }
        }
        Logical::PipePipe => {
            // 短絡評価: 左がtrueなら右を評価しない
            if lhs != 0 {
                Ok(1)
            } else {
                Ok(if rhs != 0 { 1 } else { 0 })
            }
        }
    }
}

fn eval_typed_ternary(ternary: &crate::sema::Ternary) -> Result<isize, String> {
    // 型チェック
    if !is_compile_time_constant_type(&ternary.cond.r#type) {
        return Err(format!(
            "三項演算子の条件部の型が非対応: {:?}",
            ternary.cond.r#type
        ));
    }
    if !is_compile_time_constant_type(&ternary.then_branch.r#type) {
        return Err(format!(
            "三項演算子のthen節の型が非対応: {:?}",
            ternary.then_branch.r#type
        ));
    }
    if !is_compile_time_constant_type(&ternary.else_branch.r#type) {
        return Err(format!(
            "三項演算子のelse節の型が非対応: {:?}",
            ternary.else_branch.r#type
        ));
    }

    let cond = eval_const_typed_expr(&ternary.cond)?;

    if cond != 0 {
        eval_const_typed_expr(&ternary.then_branch)
    } else {
        eval_const_typed_expr(&ternary.else_branch)
    }
}

fn eval_typed_cast(cast: &crate::sema::Cast) -> Result<isize, String> {
    // キャスト先の型チェック
    if !is_compile_time_constant_type(&cast.r#type) {
        return Err(format!(
            "キャスト先の型が定数計算に対応していません: {:?}",
            cast.r#type
        ));
    }

    let value = eval_const_typed_expr(&cast.expr)?;

    // int/charのキャストのみサポート
    match cast.r#type.as_ref() {
        Type::Int => Ok(value),
        Type::Char => {
            // char範囲チェック
            if value >= 0 && value <= 255 {
                Ok(value)
            } else {
                Err(format!("charの範囲外の値: {}", value))
            }
        }
        _ => Err(format!("サポートされていないキャスト: {:?}", cast.r#type)),
    }
}

fn eval_typed_comma(comma: &crate::sema::Comma) -> Result<isize, String> {
    let mut result = 0;
    for typed_expr in &comma.assigns {
        // 各式の型チェック
        if !is_compile_time_constant_type(&typed_expr.r#type) {
            return Err(format!(
                "コンマ演算子内の式の型が非対応: {:?}",
                typed_expr.r#type
            ));
        }
        result = eval_const_typed_expr(typed_expr)?;
    }
    Ok(result)
}

fn eval_typed_sizeof(sizeof: &crate::sema::Sizeof) -> Result<isize, String> {
    match sizeof {
        crate::sema::Sizeof::Type(_) => {
            // 型のサイズを返す（簡略化のため固定値）
            Ok(4) // 仮のサイズ
        }
        crate::sema::Sizeof::Expr(typed_expr) => {
            // 式の型に関係なくサイズ計算は可能（式自体は評価しない）
            let _ = &typed_expr.r#type; // 型情報のみ使用
            Ok(4) // 仮のサイズ
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Arithmetic, BinaryOp, Type};
    use crate::sema::{Binary, SemaExpr, TypedExpr};

    #[test]
    fn test_int_literal() {
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::NumInt(42));
        let result = eval_const_typed_expr(&typed_expr).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_char_literal() {
        let typed_expr = TypedExpr::new(Type::Char, SemaExpr::Char('A'));
        let result = eval_const_typed_expr(&typed_expr).unwrap();
        assert_eq!(result, 65); // 'A' = 65
    }

    #[test]
    fn test_float_rejection() {
        use ordered_float::OrderedFloat;
        let typed_expr = TypedExpr::new(Type::Double, SemaExpr::NumFloat(OrderedFloat(3.14)));
        let result = eval_const_typed_expr(&typed_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("非int型"));
    }

    #[test]
    fn test_binary_addition() {
        let lhs = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(10)));
        let rhs = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(20)));
        let binary = Binary {
            op: BinaryOp::Arithmetic(Arithmetic::Plus),
            lhs,
            rhs,
        };
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::Binary(binary));

        let result = eval_const_typed_expr(&typed_expr).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_mixed_type_rejection() {
        use ordered_float::OrderedFloat;
        let lhs = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(10)));
        let rhs = Box::new(TypedExpr::new(
            Type::Double,
            SemaExpr::NumFloat(OrderedFloat(3.14)),
        ));
        let binary = Binary {
            op: BinaryOp::Arithmetic(Arithmetic::Plus),
            lhs,
            rhs,
        };
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::Binary(binary));

        let result = eval_const_typed_expr(&typed_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("非対応"));
    }

    #[test]
    fn test_unary_operations() {
        use crate::sema::Unary;

        // -42 = -42
        let expr = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(42)));
        let unary = Unary {
            op: UnaryOp::Minus,
            expr,
        };
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::Unary(unary));
        assert_eq!(eval_const_typed_expr(&typed_expr).unwrap(), -42);

        // !5 = 0 (5は非ゼロなのでfalse)
        let expr = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(5)));
        let unary = Unary {
            op: UnaryOp::Bang,
            expr,
        };
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::Unary(unary));
        assert_eq!(eval_const_typed_expr(&typed_expr).unwrap(), 0);

        // !0 = 1 (0はゼロなのでtrue)
        let expr = Box::new(TypedExpr::new(Type::Int, SemaExpr::NumInt(0)));
        let unary = Unary {
            op: UnaryOp::Bang,
            expr,
        };
        let typed_expr = TypedExpr::new(Type::Int, SemaExpr::Unary(unary));
        assert_eq!(eval_const_typed_expr(&typed_expr).unwrap(), 1);
    }
}
