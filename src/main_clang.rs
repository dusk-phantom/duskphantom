extern crate compiler;

#[cfg(feature = "clang_frontend")]
use compiler::compile_clang;
use compiler::{compile, config, errors::handle_error, get_args};
use std::borrow::Borrow;
fn main() {
    #[cfg(feature = "clang_frontend")]
    {
        let (sy_path, output_path, opt_flag, asm_flag) = get_args();
        let result = compile_clang(&sy_path, &output_path, opt_flag, asm_flag);
        if let Err(err) = result.borrow() {
            handle_error(&err);
        }
    }
}
