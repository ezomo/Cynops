//そもそusize以外は計算する必要がない
use crate::ast::{Arithmetic, BinaryOp, Comparison, Logical, UnaryOp};
use crate::ast::{Binary, Cast, Comma, Expr, Sizeof, Ternary, Unary};
use ordered_float::OrderedFloat;

#[derive(Debug, PartialEq, Clone)]
pub enum ConstValue {
    Int(isize),
    Float(OrderedFloat<f64>),
    Char(char),
    String(String),
}

impl ConstValue {
    pub fn to_bool(&self) -> bool {
        match self {
            ConstValue::Int(n) => *n != 0,
            ConstValue::Float(f) => f.0 != 0.0,
            ConstValue::Char(c) => *c != '\0',
            ConstValue::String(s) => !s.is_empty(),
        }
    }

    pub fn to_int(&self) -> Result<isize, String> {
        match self {
            ConstValue::Int(n) => Ok(*n),
            ConstValue::Float(f) => Ok(f.0 as isize),
            ConstValue::Char(c) => Ok(*c as isize),
            ConstValue::String(_) => Err("Cannot convert string to integer".to_string()),
        }
    }
}

pub fn eval_const_expr(expr: &Expr) -> Result<ConstValue, String> {
    match expr {
        Expr::NumInt(n) => Ok(ConstValue::Int(*n as isize)),
        Expr::NumFloat(f) => Ok(ConstValue::Float(*f)),
        Expr::Char(c) => Ok(ConstValue::Char(*c)),
        Expr::String(s) => Ok(ConstValue::String(s.clone())),

        Expr::Unary(unary) => eval_unary(unary),
        Expr::Binary(binary) => eval_binary(binary),
        Expr::Ternary(ternary) => eval_ternary(ternary),
        Expr::Cast(cast) => eval_cast(cast),
        Expr::Comma(comma) => eval_comma(comma),
        Expr::Sizeof(sizeof) => eval_sizeof(sizeof),

        // 実行時要素は処理しない
        Expr::Variable(_) => Err("Variables are not compile-time constants".to_string()),
        Expr::Call(_) => Err("Function calls are not compile-time constants".to_string()),
        Expr::Subscript(_) => Err("Array subscripts are not compile-time constants".to_string()),
        Expr::MemberAccess(_) => Err("Member access is not compile-time constant".to_string()),
        Expr::Postfix(_) => Err("Postfix operations are not compile-time constants".to_string()),
        Expr::Assign(_) => Err("Assignment is not compile-time constant".to_string()),
    }
}

fn eval_unary(unary: &Unary) -> Result<ConstValue, String> {
    let operand = eval_const_expr(&unary.expr)?;

    match unary.op {
        UnaryOp::Minus => match operand {
            ConstValue::Int(n) => Ok(ConstValue::Int(-n)),
            ConstValue::Float(f) => Ok(ConstValue::Float(-f)),
            _ => Err("Cannot apply unary minus to non-numeric value".to_string()),
        },
        UnaryOp::Bang => Ok(ConstValue::Int(if operand.to_bool() { 0 } else { 1 })),
        UnaryOp::Tilde => match operand {
            ConstValue::Int(n) => Ok(ConstValue::Int(!n)),
            ConstValue::Char(c) => Ok(ConstValue::Int(!(c as u8 as isize))),
            _ => Err("Cannot apply bitwise NOT to non-integer value".to_string()),
        },
        UnaryOp::Ampersand => Err("Address-of operator is not compile-time constant".to_string()),
        UnaryOp::Asterisk => Err("Dereference operator is not compile-time constant".to_string()),
        UnaryOp::PlusPlus => Err("Pre-increment is not compile-time constant".to_string()),
        UnaryOp::MinusMinus => Err("Pre-decrement is not compile-time constant".to_string()),
    }
}

fn eval_binary(binary: &Binary) -> Result<ConstValue, String> {
    let lhs = eval_const_expr(&binary.lhs)?;
    let rhs = eval_const_expr(&binary.rhs)?;

    match binary.op {
        BinaryOp::Arithmetic(arith) => eval_arithmetic(arith, lhs, rhs),
        BinaryOp::Comparison(comp) => eval_comparison(comp, lhs, rhs),
        BinaryOp::Logical(logical) => eval_logical(logical, lhs, rhs),
    }
}

fn eval_arithmetic(op: Arithmetic, lhs: ConstValue, rhs: ConstValue) -> Result<ConstValue, String> {
    match op {
        Arithmetic::Plus => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a + b)),
            (ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a + b)),
            (ConstValue::Int(a), ConstValue::Float(b)) => {
                Ok(ConstValue::Float(OrderedFloat(a as f64) + b))
            }
            (ConstValue::Float(a), ConstValue::Int(b)) => {
                Ok(ConstValue::Float(a + OrderedFloat(b as f64)))
            }
            _ => Err("Invalid operands for addition".to_string()),
        },
        Arithmetic::Minus => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a - b)),
            (ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a - b)),
            (ConstValue::Int(a), ConstValue::Float(b)) => {
                Ok(ConstValue::Float(OrderedFloat(a as f64) - b))
            }
            (ConstValue::Float(a), ConstValue::Int(b)) => {
                Ok(ConstValue::Float(a - OrderedFloat(b as f64)))
            }
            _ => Err("Invalid operands for subtraction".to_string()),
        },
        Arithmetic::Asterisk => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a * b)),
            (ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a * b)),
            (ConstValue::Int(a), ConstValue::Float(b)) => {
                Ok(ConstValue::Float(OrderedFloat(a as f64) * b))
            }
            (ConstValue::Float(a), ConstValue::Int(b)) => {
                Ok(ConstValue::Float(a * OrderedFloat(b as f64)))
            }
            _ => Err("Invalid operands for multiplication".to_string()),
        },
        Arithmetic::Slash => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => {
                if b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(ConstValue::Int(a / b))
                }
            }
            (ConstValue::Float(a), ConstValue::Float(b)) => {
                if b.0 == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(ConstValue::Float(a / b))
                }
            }
            (ConstValue::Int(a), ConstValue::Float(b)) => {
                if b.0 == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(ConstValue::Float(OrderedFloat(a as f64) / b))
                }
            }
            (ConstValue::Float(a), ConstValue::Int(b)) => {
                if b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(ConstValue::Float(a / OrderedFloat(b as f64)))
                }
            }
            _ => Err("Invalid operands for division".to_string()),
        },
        Arithmetic::Percent => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => {
                if b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(ConstValue::Int(a % b))
                }
            }
            _ => Err("Modulo operation requires integer operands".to_string()),
        },
        Arithmetic::Ampersand => {
            let a = lhs.to_int()?;
            let b = rhs.to_int()?;
            Ok(ConstValue::Int(a & b))
        }
        Arithmetic::Pipe => {
            let a = lhs.to_int()?;
            let b = rhs.to_int()?;
            Ok(ConstValue::Int(a | b))
        }
        Arithmetic::Caret => {
            let a = lhs.to_int()?;
            let b = rhs.to_int()?;
            Ok(ConstValue::Int(a ^ b))
        }
        Arithmetic::LessLess => {
            let a = lhs.to_int()?;
            let b = rhs.to_int()?;
            if b < 0 || b >= 64 {
                Err("Invalid shift amount".to_string())
            } else {
                Ok(ConstValue::Int(a << b))
            }
        }
        Arithmetic::GreaterGreater => {
            let a = lhs.to_int()?;
            let b = rhs.to_int()?;
            if b < 0 || b >= 64 {
                Err("Invalid shift amount".to_string())
            } else {
                Ok(ConstValue::Int(a >> b))
            }
        }
    }
}

fn eval_comparison(op: Comparison, lhs: ConstValue, rhs: ConstValue) -> Result<ConstValue, String> {
    let result = match op {
        Comparison::EqualEqual => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a == b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a == b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a == b,
            (ConstValue::String(a), ConstValue::String(b)) => a == b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) == b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a == OrderedFloat(b as f64),
            _ => false,
        },
        Comparison::NotEqual => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a != b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a != b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a != b,
            (ConstValue::String(a), ConstValue::String(b)) => a != b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) != b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a != OrderedFloat(b as f64),
            _ => true,
        },
        Comparison::Less => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a < b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a < b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a < b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) < b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a < OrderedFloat(b as f64),
            _ => return Err("Invalid operands for less-than comparison".to_string()),
        },
        Comparison::LessEqual => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a <= b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a <= b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a <= b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) <= b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a <= OrderedFloat(b as f64),
            _ => return Err("Invalid operands for less-than-or-equal comparison".to_string()),
        },
        Comparison::Greater => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a > b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a > b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a > b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) > b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a > OrderedFloat(b as f64),
            _ => return Err("Invalid operands for greater-than comparison".to_string()),
        },
        Comparison::GreaterEqual => match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a >= b,
            (ConstValue::Float(a), ConstValue::Float(b)) => a >= b,
            (ConstValue::Char(a), ConstValue::Char(b)) => a >= b,
            (ConstValue::Int(a), ConstValue::Float(b)) => OrderedFloat(a as f64) >= b,
            (ConstValue::Float(a), ConstValue::Int(b)) => a >= OrderedFloat(b as f64),
            _ => return Err("Invalid operands for greater-than-or-equal comparison".to_string()),
        },
    };

    Ok(ConstValue::Int(if result { 1 } else { 0 }))
}

fn eval_logical(op: Logical, lhs: ConstValue, rhs: ConstValue) -> Result<ConstValue, String> {
    match op {
        Logical::AmpersandAmpersand => {
            // 短絡評価: 左がfalseなら右を評価しない
            if !lhs.to_bool() {
                Ok(ConstValue::Int(0))
            } else {
                Ok(ConstValue::Int(if rhs.to_bool() { 1 } else { 0 }))
            }
        }
        Logical::PipePipe => {
            // 短絡評価: 左がtrueなら右を評価しない
            if lhs.to_bool() {
                Ok(ConstValue::Int(1))
            } else {
                Ok(ConstValue::Int(if rhs.to_bool() { 1 } else { 0 }))
            }
        }
    }
}

fn eval_ternary(ternary: &Ternary) -> Result<ConstValue, String> {
    let cond = eval_const_expr(&ternary.cond)?;

    if cond.to_bool() {
        eval_const_expr(&ternary.then_branch)
    } else {
        eval_const_expr(&ternary.else_branch)
    }
}

fn eval_cast(cast: &Cast) -> Result<ConstValue, String> {
    let value = eval_const_expr(&cast.expr)?;

    // 簡単なキャスト処理（実際の型情報が必要だが、ここでは簡略化）
    match value {
        ConstValue::Int(n) => Ok(ConstValue::Int(n)),
        ConstValue::Float(f) => Ok(ConstValue::Float(f)),
        ConstValue::Char(c) => Ok(ConstValue::Char(c)),
        ConstValue::String(s) => Ok(ConstValue::String(s)),
    }
}

fn eval_comma(comma: &Comma) -> Result<ConstValue, String> {
    let mut result = ConstValue::Int(0);
    for expr in &comma.assigns {
        result = eval_const_expr(expr)?;
    }
    Ok(result)
}

fn eval_sizeof(sizeof: &Sizeof) -> Result<ConstValue, String> {
    match sizeof {
        Sizeof::Type(_) => {
            // 型のサイズを返す（簡略化のため固定値）
            Ok(ConstValue::Int(4)) // 仮のサイズ
        }
        Sizeof::Expr(expr) => {
            // 式の型からサイズを推定（簡略化）
            let _ = eval_const_expr(expr)?;
            Ok(ConstValue::Int(4)) // 仮のサイズ
        }
    }
}
