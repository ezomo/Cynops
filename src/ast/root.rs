use super::Block;
use super::FunctionSig;
use super::Stmt;
use super::decl::FunctionDef;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionProto {
    pub sig: FunctionSig,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    FunctionDef(FunctionDef),
    FunctionProto(FunctionProto),
    Stmt(Stmt),
}

impl Program {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl TopLevel {
    pub fn function_def(function_sig: FunctionSig, stmt: Block) -> Self {
        TopLevel::FunctionDef(FunctionDef {
            sig: function_sig,
            body: stmt,
        })
    }

    pub fn stmt(stmt: Stmt) -> Self {
        TopLevel::Stmt(stmt)
    }

    pub fn function_proto(function_sig: FunctionSig) -> Self {
        TopLevel::FunctionProto(FunctionProto { sig: function_sig })
    }
}
