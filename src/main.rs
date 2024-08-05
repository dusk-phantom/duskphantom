// use self lib
extern crate compiler;

use std::borrow::Borrow;

use clap::Parser;
use compiler::{args::Cli, compile, errors::handle_error};

fn main() {
    let cli = Cli::parse();
    start_compiler(&cli);
}
fn start_compiler(cli: &Cli) {
    let (sy_path, output_path, opt_flag, asm_flag, ll_path) = (
        &cli.sy,
        &cli.output,
        cli.optimize != 0,
        cli.asm,
        cli.ll.clone(),
    );
    let result = compile(sy_path, output_path, opt_flag, asm_flag, ll_path);
    if let Err(err) = result.borrow() {
        handle_error(err);
    }
}
