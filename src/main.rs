// use self lib
extern crate compiler;

use clap::{App, Arg};

fn main() {
    let src_arg = Arg::new("src").help("Source file").required(true).index(1);
    let output_arg = Arg::new("output path")
        .short('o')
        .value_name("OUTPUT_FILE")
        .help("output asm file")
        .default_value("a.s")
        .takes_value(true)
        .required(true);
    let opt_arg = Arg::new("opt")
        .short('O')
        .long("optimized")
        .value_name("optimization level")
        .help("optimization level")
        .takes_value(false)
        .required(false);
    let asm_flag = Arg::new("asm")
        .short('S')
        .long("asm")
        .value_name("ASM_FILE")
        .help("output asm file")
        .takes_value(false)
        .required(true);
    // use clap to define a cli
    let matches = App::new("compiler")
        .version("1.0")
        .author("compilerhit")
        .about("optimized compiler of sysy")
        .args(&[src_arg, asm_flag, output_arg, opt_arg])
        .get_matches();

    // get src file from first arg after app name
    let src_file = matches.value_of("src").expect("msg: src file not found");
    // show if add -S,which is must specified
    let asm_flag = matches.is_present("asm");
    // get output file from -o
    let output_file = matches
        .value_of("output path")
        .expect("output file not found");
    // get opt level
    let opt_flag = matches.is_present("opt");
    // read src file
    let src = std::fs::read_to_string(src_file).expect("msg: read src file failed");
    // compile
    let asm = compiler::compile(&src, opt_flag).expect("msg: compile failed");
    // write asm file
    std::fs::write(output_file, asm).expect("msg: write asm file failed");
}
