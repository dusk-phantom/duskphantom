extern  crate compiler;



#[cfg(feature = "clang_embeded")]
use compiler::compile_clang;

use compiler::{  errors::handle_error, get_args};
use std::borrow::Borrow;

#[cfg(feature = "clang_embeded")]
use compiler::compile_clang_llc;


pub fn main() {
    #[cfg(feature = "clang_embeded")]
    {
        let (sy_path, output_path, opt_flag, asm_flag) = get_args();
        let result = compile_clang_llc(&sy_path, &output_path, opt_flag, asm_flag);
        if let Err(err) = result.borrow() {
            handle_error(&err);
        }
    }
}