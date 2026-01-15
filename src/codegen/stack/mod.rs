#[cfg(test)]
pub mod exec;
pub mod inst;

#[cfg(test)]
pub use exec::*;

pub use inst::*;

pub use super::*;

pub type Word = u16;
