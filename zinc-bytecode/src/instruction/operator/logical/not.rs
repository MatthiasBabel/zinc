//!
//! The 'logical NOT' instruction.
//!

use std::fmt;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::instruction::Instruction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Not;

impl Not {
    pub fn is_debug(&self) -> bool {
        false
    }
}

impl Into<Instruction> for Not {
    fn into(self) -> Instruction {
        Instruction::Not(self)
    }
}

impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not")
    }
}
