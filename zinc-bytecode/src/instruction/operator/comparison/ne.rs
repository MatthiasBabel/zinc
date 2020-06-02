//!
//! The 'not equals comparison' instruction.
//!

use std::fmt;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::instruction::Instruction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ne;

impl Ne {
    pub fn is_debug(&self) -> bool {
        false
    }

    pub fn wrap(self) -> Instruction {
        Instruction::Ne(self)
    }
}

impl Into<Instruction> for Ne {
    fn into(self) -> Instruction {
        Instruction::Ne(self)
    }
}

impl fmt::Display for Ne {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ne")
    }
}
