use super::*;

#[derive(Debug)]
pub struct Args {
    pub sy_path: Option<String>,
    pub output_path: String,
    pub opt_flag: bool,
    pub asm_flag: bool,
    pub ll_path: Option<String>,
}

pub fn get_args() -> Args {
    let matches = App::new("compiler")
        .about("compiler <src> [-S] [-o <output>] [-O]")
        .arg(arg!(<src> "Source file").index(1))
        .arg(arg!(-S --asm "output asm file"))
        .arg(arg!(-o --output <FILE> "output asm file").default_value("a.s"))
        .arg(arg!(-O --optimized "optimization level"))
        .arg(arg!(-l --llvm <FILE> "output llvm ir file"))
        .get_matches();
    let sy_path = matches.value_of("src").map(|s| s.to_string());
    let output_path = matches.value_of("output").unwrap().to_string();
    let opt_flag = matches.is_present("optimized");
    let asm_flag = matches.is_present("asm");
    let ll_path = matches.value_of("llvm").map(|s| s.to_string());
    Args {
        sy_path,
        output_path,
        opt_flag,
        asm_flag,
        ll_path,
    }
}

pub fn get_sy_out_opt_asm(args: &Args) -> (String, String, bool, bool) {
    (
        args.sy_path.clone().unwrap(),
        args.output_path.clone(),
        args.opt_flag,
        args.asm_flag,
    )
}
