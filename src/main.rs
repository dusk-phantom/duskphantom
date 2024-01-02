// use self lib
extern crate compiler;

use std::borrow::Borrow;

use clap::{arg, App};
use compiler::errors::handle_error;

fn main() {
    let app = App::new("compiler")
        .arg(arg!(<src> "Source file").index(1))
        .arg(arg!(-S --asm "output asm file"))
        .arg(arg!(-o --output <FILE> "output asm file"))
        .arg(arg!(-O --optimized "optimization level"))
        .get_matches();
    let src_file = app.value_of("src").expect("msg: src file not found");
    let asm_flag = app.is_present("asm");
    let output_file = app.value_of("output").unwrap_or("a.s");
    let opt_flag = app.is_present("optimized");
    let src = std::fs::read_to_string(src_file).expect("msg: read src file failed");
    let asm = compiler::compile(&src, opt_flag);
    if let Err(err) = asm.borrow() {
        handle_error(&err);
    }
    if asm_flag {
        std::fs::write(output_file, asm.unwrap()).expect("msg: write asm file failed");
    } else {
        let bin = compiler::backend::asm2bin(asm.unwrap());
        std::fs::write(output_file, bin).expect("msg: write bin file failed");
    }
}
