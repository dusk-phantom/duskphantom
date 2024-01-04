use std::env;

// tools supporting gening rv64gc assemble
pub struct Rv64gcGen;
impl Rv64gcGen {
    #[inline]
    fn gen_suffix() -> String {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let mut ret = String::with_capacity(64);
        ret.push_str(format!(".ident\t\"compiler: (visionfive2) {}\"\n", VERSION).as_str());
        ret.push_str(r#".section	.note.GNU-stack,"",@progbits"#);
        ret
    }
    #[inline]
    fn gen_preffix(file: &str) -> String {
        let mut ret = String::with_capacity(64);
        ret.push_str(format!(".file \"{}\"\n", file).as_str());
        ret.push_str(".option pic\n");
        ret.push_str(
            r#".attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0""#,
        );
        ret.push_str("\n");
        ret.push_str(".attribute unaligned_access, 0\n");
        ret.push_str(".attribute stack_align, 16");
        ret
    }
    pub fn gen_prog(file: &str, global: &str, funcs: &str) -> String {
        let mut ret = String::with_capacity(1024);
        // gen preffix
        ret.push_str(Rv64gcGen::gen_preffix(file).as_str());
        ret.push('\n');
        // gen global data
        ret.push_str(global);
        ret.push('\n');
        // gen code
        ret.push_str(funcs);
        ret.push('\n');
        // gen suffix
        ret.push_str(Rv64gcGen::gen_suffix().as_str());
        ret.push('\n');
        ret
    }
    pub fn gen_func(fname: &str, entry_bb: &str, other_bbs: &str) -> String {
        let mut ret = String::with_capacity(1024);
        ret.push_str(
            format!(
                ".text\n.align  2\n.globl  {}\n.type   {}, @function\n",
                fname, fname
            )
            .as_str(),
        );
        ret.push_str(fname);
        ret.push_str(":\n");
        ret.push_str(entry_bb);
        ret.push('\n');
        ret.push_str(other_bbs);
        ret.push('\n');
        ret.push_str(format!(".size   {}, .-{}", fname, fname).as_str());
        ret
    }
    pub fn gen_bb(label: &str, insts: &str) -> String {
        let mut ret = String::with_capacity(1024);
        ret.push_str(label);
        ret.push_str(":\n");
        ret.push_str(insts);
        ret
    }
    pub fn gen_word(name: &str, val: u32) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_dword(name: &str, val: u64) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_float(name: &str, val: f32) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_const_str(name: &str, val: &str) -> String {
        let mut ret = String::with_capacity(32 + val.len());
        ret.push_str(".globl\t");
        ret.push_str(name);
        ret.push('\n');
        ret.push_str(".section\t.rodata\n");
        ret.push_str(".align  3\n");
        ret.push_str(name);
        ret.push_str(":\n");
        ret.push_str(".string \"");
        ret.push_str(val);
        ret.push_str("\"\n");
        ret
    }
    pub fn gen_byte_arr(name: &str, val: &str) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_word_arr(name: &str, val: &str) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_dword_arr(name: &str, val: &str) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_f32_arr(name: &str, elem_num: u32, init: &[(u32, f32)]) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
    pub fn gen_f64_arr(name: &str, elem_num: u32, init: &[(u32, f64)]) -> String {
        let mut ret = String::with_capacity(128);
        ret
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gen_const_str() {
        let s = super::Rv64gcGen::gen_const_str("hello", "world");
        let raw_match = r##".globl	hello
.section	.rodata
.align  3
hello:
.string "world"
"##;
        assert_eq!(s, raw_match);
    }
}