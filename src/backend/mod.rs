mod from;
pub mod gen_asm;
pub mod irs;
pub mod optimize;
mod phisicalize;

use rayon::prelude::*;

pub use crate::context;
pub use crate::errors::BackendError;
pub use anyhow::{anyhow, Context, Result};
pub use from::*;
// pub use irs::*;

pub use optimize::*;
pub use phisicalize::*;
