use errors::CompilerError;

pub mod backend;
pub mod errors;
pub mod frontend;
pub mod middle;

pub fn compile(sysy_src: &str, opt: bool) -> Result<String, CompilerError> {
    let mut program = frontend::parse(sysy_src)?;
    if opt {
        frontend::optimize(&mut program)
    }
    let mut program = middle::gen(&mut program)?;
    if opt {
        middle::optimize(&mut program)
    }
    let mut program = backend::gen(&mut program)?;
    if opt {
        backend::optimize(&mut program)
    }
    let asm = program.gen_asm();
    Ok(asm)
}
