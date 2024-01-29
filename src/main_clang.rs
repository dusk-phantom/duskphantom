extern crate compiler;

use std::borrow::Borrow;

use compiler::{compile, compile_clang, config, errors::handle_error, get_args};

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
