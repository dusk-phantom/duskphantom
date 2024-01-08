use errors::CompilerError;

pub mod backend;
#[cfg(feature = "clang_frontend")]
pub mod clang_frontend;
pub mod config;
pub mod errors;
pub mod frontend;
pub mod middle;
pub mod utils;

use clap::{arg, App};

pub fn get_args() -> (String, String, bool, bool) {
    let app = App::new("compiler")
        .about("compiler <src> [-S] [-o <output>] [-O]")
        .arg(arg!(<src> "Source file").index(1))
        .arg(arg!(-S --asm "output asm file"))
        .arg(arg!(-o --output <FILE> "output asm file").default_value("a.s"))
        .arg(arg!(-O --optimized "optimization level"))
        .get_matches();
    let sy_path = app.value_of("src").expect("msg: src file not found");
    let asm_flag = app.is_present("asm");
    let output_path = app.value_of("output").unwrap();
    let opt_flag = app.is_present("optimized");
    (
        sy_path.to_string(),
        output_path.to_string(),
        opt_flag,
        asm_flag,
    )
}

/// compile sysy source code to rv64gc asm
pub fn compile(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
) -> Result<(), CompilerError> {
    let mut program = frontend::parse(sy_path)?;
    if opt_flag {
        frontend::optimize(&mut program);
    }
    let mut program = middle::gen(&mut program)?;
    if opt_flag {
        middle::optimize(&mut program);
    }
    let mut program = backend::gen(&mut program)?;
    if opt_flag {
        backend::optimize(&mut program);
    }
    let asm = program.gen_asm();
    let output = if !asm_flag {
        output_path.replace(".s", ".bin")
    } else {
        asm
    };
    std::fs::write(output_path, output).map_err(|err| CompilerError::IOError(err))
}

pub fn asm2bin(asm: String) -> String {
    backend::asm2bin(asm)
}
