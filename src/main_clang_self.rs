extern crate compiler;

fn main() {
    #[cfg(feature = "clang_enabled")]
    {
        use clap::Parser;
        use compiler::args::Cli;
        let cli = Cli::parse();
        start_compiler_cs(&cli);
    }
}
#[cfg(feature = "clang_enabled")]
fn start_compiler_cs(cli: &compiler::args::Cli) {
    use compiler::compile_clang;
    use compiler::errors::handle_error;
    use std::borrow::Borrow;
    let (sy_path, output_path, opt_flag, asm_flag, ll_path) = (
        &cli.sy,
        &cli.output,
        cli.optimize != 0,
        cli.asm,
        cli.ll.clone(),
    );
    let result = compile_clang(sy_path, output_path, opt_flag, asm_flag, ll_path);
    if let Err(err) = result.borrow() {
        handle_error(err);
    }
}
