use crate::sema;

use super::{
    Block, Ident, Typedef,
    types::{FunctionSig, Type},
};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum DeclStmt {
    InitVec(Vec<Init>),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef(Typedef),
}
impl DeclStmt {
    pub fn init_vec(vec: Vec<Init>) -> Self {
        DeclStmt::InitVec(vec)
    }

    pub fn r#struct(strct: Struct) -> Self {
        DeclStmt::Struct(strct)
    }

    pub fn union(union: Union) -> Self {
        DeclStmt::Union(union)
    }

    pub fn r#enum(enm: Enum) -> Self {
        DeclStmt::Enum(enm)
    }

    pub fn typedef(typedef: Typedef) -> Self {
        DeclStmt::Typedef(typedef)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct MemberDecl {
    pub ident: Ident,
    pub ty: Type,
}

impl MemberDecl {
    pub fn new(ident: Ident, ty: Type) -> Self {
        MemberDecl { ident, ty }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Struct {
    pub ident: Option<Ident>,
    pub member: Vec<MemberDecl>,
}

impl Struct {
    pub fn new(ident: Option<Ident>, member: Vec<MemberDecl>) -> Self {
        Struct { ident, member }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Union {
    pub ident: Option<Ident>,
    pub member: Vec<MemberDecl>,
}

impl Union {
    pub fn new(ident: Option<Ident>, member: Vec<MemberDecl>) -> Self {
        Union { ident, member }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Enum {
    pub ident: Option<Ident>,
    pub variants: Vec<EnumMember>,
}

impl Enum {
    pub fn new(ident: Option<Ident>, variants: Vec<EnumMember>) -> Self {
        Enum { ident, variants }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct EnumMember {
    pub ident: Ident,
    pub value: Option<usize>, // 明示的な値がある場合
}

impl EnumMember {
    pub fn new(ident: Ident, value: Option<usize>) -> Self {
        EnumMember { ident, value }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FunctionDef {
    pub sig: FunctionSig,
    pub param_names: Vec<Ident>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum InitData {
    Expr(sema::TypedExpr),
    Compound(Vec<InitData>), // 構造体・配列初期化子 {1, 2}
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Init {
    pub r: MemberDecl,
    pub l: Option<InitData>,
}

impl Init {
    pub fn new(r: MemberDecl, l: Option<InitData>) -> Self {
        // Option<InitData>に中身が存在する場合は型が一致しているかチェック
        if let Some(ref init_data) = l {
            if let Err(err) = Self::check_init_data_compatibility(&r.ty, init_data) {
                panic!("初期化子の型チェックエラー: {}", err);
            }
        }

        Init { r, l }
    }

    /// InitDataの型チェックを行う関数
    fn check_init_data_compatibility(
        member_type: &Type,
        init_data: &InitData,
    ) -> Result<(), String> {
        match (member_type, init_data) {
            // 単純な式の初期化
            (member_ty, InitData::Expr(typed_expr)) => {
                if member_ty == &typed_expr.r#type {
                    Ok(())
                } else {
                    Err(format!(
                        "型が完全に一致しません: {:?} vs {:?}",
                        member_ty, typed_expr.r#type
                    ))
                }
            }

            // 配列の複合初期化子
            (Type::Array(array_type), InitData::Compound(compound_list)) => {
                // 配列のサイズチェック（部分初期化は許可）
                if compound_list.len() > array_type.length {
                    return Err(format!(
                        "初期化子の要素数が配列サイズを超えています: {} > {}",
                        compound_list.len(),
                        array_type.length
                    ));
                }

                // 各要素の型チェック（指定された要素のみ）
                for (i, element) in compound_list.iter().enumerate() {
                    if let Err(err) =
                        Self::check_init_data_compatibility(&array_type.array_of, element)
                    {
                        return Err(format!("配列要素[{}]の型エラー: {}", i, err));
                    }
                }
                // 残りの要素は0で初期化される（C言語の仕様）
                Ok(())
            }

            // 構造体の複合初期化子
            (Type::Struct(struct_type), InitData::Compound(compound_list)) => {
                // 構造体のメンバー数チェック（部分初期化は許可）
                if compound_list.len() > struct_type.member.len() {
                    return Err(format!(
                        "初期化子の要素数が構造体のメンバー数を超えています: {} > {}",
                        compound_list.len(),
                        struct_type.member.len()
                    ));
                }

                // 各メンバーの型チェック（指定されたメンバーのみ）
                for (i, (element, member)) in
                    compound_list.iter().zip(&struct_type.member).enumerate()
                {
                    if let Err(err) = Self::check_init_data_compatibility(&member.ty, element) {
                        return Err(format!(
                            "構造体メンバー[{}]({})の型エラー: {}",
                            i, member.ident.name, err
                        ));
                    }
                }
                // 残りのメンバーは0で初期化される（C言語の仕様）
                Ok(())
            }

            // 共用体の複合初期化子
            (Type::Union(union_type), InitData::Compound(compound_list)) => {
                // 共用体は最初のメンバーのみ初期化可能
                if compound_list.len() > 1 {
                    return Err(format!(
                        "共用体の初期化子は1つの要素のみ許可されます: {}個指定されました",
                        compound_list.len()
                    ));
                }

                if let (Some(first_element), Some(first_member)) =
                    (compound_list.first(), union_type.member.first())
                {
                    if let Err(err) =
                        Self::check_init_data_compatibility(&first_member.ty, first_element)
                    {
                        return Err(format!(
                            "共用体メンバー({})の型エラー: {}",
                            first_member.ident.name, err
                        ));
                    }
                }
                Ok(())
            }

            // 不正な組み合わせ
            (member_ty, InitData::Compound(_)) => Err(format!(
                "型 {:?} に対して複合初期化子は使用できません",
                member_ty
            )),
        }
    }
}
