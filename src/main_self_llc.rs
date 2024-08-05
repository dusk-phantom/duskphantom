// use self lib
extern crate compiler;

fn main() {
    #[cfg(feature = "clang_enabled")]
    {
        use clap::Parser;
        use compiler::args::Cli;
        let cli = Cli::parse();
        start_compiler_sc(&cli);
    }
}
#[cfg(feature = "clang_enabled")]
fn start_compiler_sc(cli: &compiler::args::Cli) {
    use compiler::compile_self_llc;
    use compiler::errors::handle_error;
    use std::borrow::Borrow;
    let (sy_path, output_path, opt_flag, asm_flag, ll_path) = (
        &cli.sy,
        &cli.output,
        cli.optimize != 0,
        cli.asm,
        cli.ll.clone(),
    );
    let result = compile_self_llc(sy_path, output_path, opt_flag, asm_flag, ll_path);
    if let Err(err) = result.borrow() {
        handle_error(err);
    }
}
