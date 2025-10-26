use super::*;
use crate::codegen::stack::StackInst;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FastBF {
    Inst(StackInst),
    Move(isize),  // </>
    Const(isize), // +/-
    LB,           // [ Left Bracket
    RB,           // ] Right Bracket
    In,           // , input
    Out,          // . output
}

impl From<BF> for FastBF {
    fn from(value: BF) -> Self {
        use BF::*;
        use FastBF::*;
        match value {
            Profile(p) => Inst(p),
            Dbg(_) => unimplemented!(),
            Left => Move(-1),
            Right => Move(1),
            Dec => Const(-1),
            Inc => Const(1),
            LBrac => LB,
            RBrac => RB,
            Input => In,
            Output => Out,
        }
    }
}
