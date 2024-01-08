// use self lib
extern crate compiler;

use std::borrow::Borrow;

use compiler::{compile, config, errors::handle_error, get_args};

fn main() {
    let (sy_path, output_path, opt_flag, asm_flag) = get_args();
    let result = compile(&sy_path, &output_path, opt_flag, asm_flag);
    if let Err(err) = result.borrow() {
        handle_error(&err);
    }
}
