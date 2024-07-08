// use self lib
extern crate compiler;

use std::borrow::Borrow;

use compiler::{args::get_args, compile, errors::handle_error};

fn main() {
    let args = get_args();
    println!("{:?}", args);
    let (sy_path, output_path, opt_flag, asm_flag, ll_path) = (
        args.sy_path.unwrap(),
        args.output_path,
        args.opt_flag,
        args.asm_flag,
        args.ll_path,
    );
    let result = compile(&sy_path, &output_path, opt_flag, asm_flag, ll_path);
    if let Err(err) = result.borrow() {
        handle_error(err);
    }
}
