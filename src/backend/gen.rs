use std::env;

use rand::seq::index;

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
    pub fn gen_prim_var(vname: &str, size: u32, default: Option<&str>) -> String {
        let template = r#"
.text
	.globl	{1}
	.bss
	.align	3
	.type	{1}, @object
	.size	{1}, {2}
a:
	.zero	{2}
"#;
        let mut ret = String::with_capacity(256);
        ret.push_str(
            template
                .replace("{1}", vname)
                .replace("{2}", size.to_string().as_str())
                .as_str(),
        );
        match default {
            Some(d) => {
                ret.push_str(format!(".word {}\n", d).as_str());
            }
            None => {
                ret.push_str(format!(".zero {}\n", size).as_str());
            }
        }
        ret
    }
    pub fn gen_arr(arr: &str, size: u32, init: &[(u32, &str)]) -> String {
        let template = r##"
    .text
    .globl	{1}
	.data
	.align	3
	.set	.LANCHOR0,. + 0
	.type	{1}, @object
	.size	{1}, {2}
{1}:
"##;
        let mut ret = String::new();
        ret.push_str(
            template
                .replace("{1}", arr)
                .replace("{2}", size.to_string().as_str())
                .as_str(),
        );
        let mut init: Vec<(u32, &str)> = init.to_vec();
        init.sort_by(|a, b| a.0.cmp(&b.0));
        let mut last = 0;
        while last < init.len() {
            let (idx, val) = init[last];
            if last == 0 && idx != 0 {
                ret.push_str(format!(".zero {}\n", idx * 4).as_str());
            }
            ret.push_str(format!(".word {}\n", val).as_str());
            if last == init.len() - 1 && idx * 4 < size {
                ret.push_str(format!(".zero {}\n", size - (idx + 1) * 4).as_str());
            }
            last += 1;
        }
        ret
    }
    pub fn gen_str() {}
}
