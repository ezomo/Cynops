use crate::sema::ast::*;

pub trait Size {
    fn size(&self) -> usize;
}

impl Size for Type {
    fn size(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Error => 0,
            Type::Char => 1,
            Type::Int => 1,
            // 普通に2と書いても良かったがのちに困るので
            Type::Double => &Type::Int.size() + &Type::Int.size() + &Type::Int.size(),
            Type::DotDotDot => 0,
            Type::Unresolved => 0,
            Type::Pointer(_) => 1,
            Type::Func(f) => f.size(),      // 関数型のサイズ取得
            Type::Array(arr) => arr.size(), // 配列のサイズ取得
            Type::Struct(s) => s.size(),    // 構造体のサイズ取得
            Type::Union(u) => u.size(),     // 共用体のサイズ取得
            Type::Enum(e) => e.size(),      // enumのサイズ取得
            Type::Typedef(this) => this.size(),
        }
    }
}

impl Size for Func {
    fn size(&self) -> usize {
        Type::Int.size() // 関数型は Int と仮定アドレス幅
    }
}

impl Size for Array {
    fn size(&self) -> usize {
        let element_size = self.array_of.size();
        match &self.length {
            Some(len_expr) => element_size * len_expr.consume_const() as usize,
            None => panic!("Incomplete array type"),
        }
    }
}

impl Size for Struct {
    fn size(&self) -> usize {
        self.member
            .iter()
            .map(|x| x.get_type().unwrap().size())
            .sum()
    }
}

impl Size for Union {
    fn size(&self) -> usize {
        self.member
            .iter()
            .map(|x| x.get_type().unwrap().size())
            .max()
            .unwrap_or(0)
    }
}

impl Size for Enum {
    fn size(&self) -> usize {
        Type::Int.size() // enumはintと同じサイズ
    }
}

impl Size for Symbol {
    fn size(&self) -> usize {
        self.get_type().unwrap().size()
    }
}
