extern crate compiler;

fn main() {
    #[cfg(feature = "clang_embeded")]
    {
        use compiler::compile_clang;
        use compiler::{args::*, errors::handle_error};
        use std::borrow::Borrow;
        let args = get_args();
        let (sy_path, output_path, opt_flag, asm_flag) = get_sy_out_opt_asm(&args);
        let result = compile_clang(&sy_path, &output_path, opt_flag, asm_flag, args.ll_path);
        if let Err(err) = result.borrow() {
            handle_error(err);
        }
    }
}
