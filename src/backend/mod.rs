mod from;
pub mod gen_asm;
pub mod irs;
pub mod optimize;
mod phisicalize;

use rayon::prelude::*;

pub use crate::errors::BackendError;
pub use from::*;
pub use irs::*;
pub use optimize::*;
pub use phisicalize::*;
