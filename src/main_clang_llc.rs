extern crate compiler;

pub fn main() {
    #[cfg(feature = "clang_enabled")]
    {
        use compiler::compile_clang_llc;
        use compiler::errors::handle_error;
        use std::borrow::Borrow;
        let args = compiler::args::get_args();
        let (sy_path, output_path, opt_flag, asm_flag) = compiler::args::get_sy_out_opt_asm(&args);
        let result = compile_clang_llc(&sy_path, &output_path, opt_flag, asm_flag, args.ll_path);
        if let Err(err) = result.borrow() {
            handle_error(err);
        }
    }
}
