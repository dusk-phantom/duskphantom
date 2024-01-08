extern crate compiler;

fn main() {
    let (src_file, output_file, opt_flag, asm_flag) = compiler::get_args();
    #[cfg(feature = "clang_frontend")]
    {
        let program = compiler::clang_frontend::Program::parse(src_file);
        let program = compiler::backend::clang_gen(&program);
        if let Err(err) = program.borrow() {
            compiler::errors::handle_error(&err);
        }
        if opt_flag {
            compiler::backend::optimize(program.unwrap().borrow_mut());
        }
        let asm = program.unwrap().gen_asm();
        if asm_flag {
            std::fs::write(output_file, asm.unwrap()).expect("msg: write asm file failed");
        } else {
            let bin = compiler::backend::asm2bin(asm.unwrap());
            std::fs::write(output_file, bin).expect("msg: write bin file failed");
        }
    }
}
