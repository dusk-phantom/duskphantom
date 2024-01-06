extern crate compiler;

use clap::{arg, App};
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
