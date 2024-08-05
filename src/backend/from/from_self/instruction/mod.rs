mod call;
mod gep;
mod normal;

use std::collections::HashMap;

pub use super::*;

pub use builder::IRBuilder;

pub use crate::utils::mem::ObjPtr;

pub use crate::{backend::*, ssa2tac_three_float, ssa2tac_three_usual_Itype};
pub use crate::{context, middle};

pub use crate::middle::ir::instruction::binary_inst::BinaryInst;
pub use crate::middle::ir::instruction::downcast_ref;
pub use crate::middle::ir::Instruction;
pub use anyhow::{Context, Result};
pub use var::FloatVar;
