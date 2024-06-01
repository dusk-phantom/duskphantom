use std::env;

// 为各种基础的 数据类型以及其值的表达实现基本的
pub trait Data {
    fn size() -> u32;
    fn to_str(&self) -> String;
}
impl Data for u8 {
    fn size() -> u32 {
        1
    }
    fn to_str(&self) -> String {
        format!(".byte\t0x{:X}", self)
    }
}
impl Data for u16 {
    fn size() -> u32 {
        2
    }
    fn to_str(&self) -> String {
        format!(".short\t0x{:X}", self)
    }
}
impl Data for u32 {
    fn size() -> u32 {
        4
    }
    fn to_str(&self) -> String {
        format!(".word\t0x{:X}", self)
    }
}
impl Data for f32 {
    fn size() -> u32 {
        4
    }
    fn to_str(&self) -> String {
        format!(".float\t{}", self)
    }
}
impl Data for u64 {
    fn size() -> u32 {
        8
    }
    fn to_str(&self) -> String {
        format!(".dword\t0x{:X}", self)
    }
}
impl Data for f64 {
    fn size() -> u32 {
        8
    }
    fn to_str(&self) -> String {
        format!(".double\t{}", self)
    }
}

// tools supporting gening rv64gc assemble
pub struct GenTool;
impl GenTool {
    #[inline]
    fn gen_suffix() -> String {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let mut ret = String::with_capacity(64);
        ret.push_str(format!(".ident\t\"compiler: (visionfive2) {}\"\n", VERSION).as_str());
        ret.push_str(r#".section	.note.GNU-stack,"",@progbits"#);
        ret
    }
    #[inline]
    fn gen_prefix(file: &str) -> String {
        let mut ret = String::with_capacity(64);
        ret.push_str(format!(".file \"{}\"\n", file).as_str());
        ret.push_str(".option pic\n");
        ret.push_str(
            r#".attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0""#,
        );
        ret.push('\n');
        ret.push_str(".attribute unaligned_access, 0\n");
        ret.push_str(".attribute stack_align, 16");
        ret
    }
    #[inline]
    pub fn gen_prog(file: &str, global: &str, funcs: &str) -> String {
        let mut ret = String::with_capacity(1024);
        // gen prefix
        ret.push_str(GenTool::gen_prefix(file).as_str());
        ret.push('\n');
        // gen global data
        ret.push_str(global);
        ret.push('\n');
        // gen code
        ret.push_str(funcs);
        ret.push('\n');
        // gen suffix
        ret.push_str(GenTool::gen_suffix().as_str());
        ret.push('\n');
        ret
    }
    #[inline]
    pub fn gen_func(fname: &str, entry_bb: &str, other_bbs: &str) -> String {
        let mut ret = String::with_capacity(1024);
        ret.push_str(".text\n.align\t3\n");
        ret.push_str(format!(".globl\t{}\n", fname).as_str());
        ret.push_str(format!(".type\t{}, @function\n", fname).as_str());
        ret.push_str(fname);
        ret.push_str(":\n");
        ret.push_str(entry_bb);
        ret.push('\n');
        ret.push_str(other_bbs);
        ret.push('\n');
        ret.push_str(format!(".size\t{}, .-{}", fname, fname).as_str());
        ret
    }
    #[inline]
    pub fn gen_bb(label: &str, insts: &str) -> String {
        let mut ret = String::with_capacity(1024);
        ret.push_str(label);
        ret.push_str(":\n");
        ret.push_str(insts);
        ret
    }
    #[inline]
    pub fn gen_word(name: &str, val: u32) -> String {
        let mut ret = String::with_capacity(64);
        ret.push_str(".data\n.align\t3\n");
        ret.push_str(format!(".globl\t{}\n", name).as_str());
        ret.push_str(
            format!(
                ".type\t{0}, @object\n.size\t{0}, 4\n{0}:\n.word\t0x{1:X}",
                name, val
            )
            .as_str(),
        );
        ret
    }
    #[inline]
    pub fn gen_dword(name: &str, val: u64) -> String {
        let mut ret = String::with_capacity(64);
        ret.push_str(".data\n.align\t3\n");
        ret.push_str(format!(".globl\t{}\n", name).as_str());
        ret.push_str(
            format!(
                ".type\t{0}, @object\n.size\t{0}, 8\n{0}:\n.dword\t0x{1:X}",
                name, val
            )
            .as_str(),
        );
        ret
    }
    #[inline]
    pub fn gen_float(name: &str, val: f32) -> String {
        let mut ret = String::with_capacity(128);
        ret.push_str(".data\n.align\t3\n");
        ret.push_str(format!(".globl\t{}\n", name).as_str());
        ret.push_str(
            format!(
                ".type\t{0}, @object\n.size\t{0}, 4\n{0}:\n.float\t{1}",
                name, val
            )
            .as_str(),
        );
        ret
    }
    #[inline]
    pub fn gen_double(name: &str, val: f64) -> String {
        let mut ret = String::with_capacity(128);
        ret.push_str(".data\n.align\t3\n");
        ret.push_str(format!(".globl\t{}\n", name).as_str());
        ret.push_str(
            format!(
                ".type\t{0}, @object\n.size\t{0}, 8\n{0}:\n.double\t{1}",
                name, val
            )
            .as_str(),
        );
        ret
    }
    #[inline]
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
    #[inline]
    pub fn gen_array<T: Data>(name: &str, num_elems: u32, init: &[(u32, T)]) -> String {
        let mut ret = String::with_capacity(128);
        let size_elem: u32 = T::size();
        ret.push_str(".data\n.align\t3\n");
        ret.push_str(format!(".globl\t{0}\n", name).as_str());
        ret.push_str(
            format!(
                ".type\t{0}, @object\n.size\t{0}, {1}\n",
                name,
                num_elems * size_elem
            )
            .as_str(),
        );
        ret.push_str(format!("{0}:\n", name).as_str());
        let mut init: Vec<&(u32, T)> = init.iter().collect();
        init.sort_by(|(idx1, _), (idx2, _)| idx1.cmp(idx2));
        for (index, (idx, val)) in init.iter().enumerate() {
            if index == 0 && idx != &0 {
                ret.push_str(format!(".zero\t{}\n", idx * size_elem).as_str());
            } else if index != 0 {
                let prev_idx = init[index - 1].0;
                if idx - prev_idx != 1 {
                    ret.push_str(format!(".zero\t{}\n", (idx - prev_idx - 1) * size_elem).as_str());
                }
            }
            ret.push_str(format!("{}\n", val.to_str()).as_str());
            if index == init.len() - 1 {
                ret.push_str(format!(".zero\t{}", (num_elems - idx - 1) * size_elem).as_str());
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gen_const_str() {
        let s = super::GenTool::gen_const_str("hello", "world");
        let raw_match = r##".globl	hello
.section	.rodata
.align  3
hello:
.string "world"
"##;
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_word() {
        let s = super::GenTool::gen_word("hello", 0x12345678);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 4
hello:
.word\t0x12345678";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_dword() {
        let s = super::GenTool::gen_dword("hello", 0x1234567890ABCDEF);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 8
hello:
.dword\t0x1234567890ABCDEF";
        assert_eq!(s, raw_match);
    }

    #[test]
    fn test_gen_float() {
        let s = super::GenTool::gen_float("hello", 1.2345678);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 4
hello:
.float\t1.2345678";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_double() {
        let s = super::GenTool::gen_double("hello", 1.234_567_890_123_456_7);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 8
hello:
.double\t1.2345678901234567";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_array() {
        let s = super::GenTool::gen_array::<u32>("hello", 10, &[(0, 1), (1, 2), (2, 3)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 40
hello:
.word\t0x1
.word\t0x2
.word\t0x3
.zero\t28";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_bb() {
        let s = super::GenTool::gen_bb("hello", "addi x0, x0, 0");
        println!("{}", s);
        let raw_match = "hello:
addi x0, x0, 0";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_func() {
        let s = super::GenTool::gen_func(
            "hello",
            "hello_0:\naddi a0, a0, 33",
            "hello_1:\naddi x0, x0, 0",
        );
        println!("{}", s);
        let raw_match = ".text
.align\t3
.globl\thello
.type\thello, @function
hello:
hello_0:
addi a0, a0, 33
hello_1:
addi x0, x0, 0
.size\thello, .-hello";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_byte_arr() {
        let s = super::GenTool::gen_array::<u8>("hello", 10, &[(0, 1), (1, 2), (2, 3)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 10
hello:
.byte\t0x1
.byte\t0x2
.byte\t0x3
.zero\t7";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_short_arr() {
        let s = super::GenTool::gen_array::<u16>("hello", 10, &[(0, 1), (1, 2), (2, 3)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 20
hello:
.short\t0x1
.short\t0x2
.short\t0x3
.zero\t14";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_word_arr() {
        let s = super::GenTool::gen_array::<u32>("hello", 10, &[(0, 1), (1, 2), (2, 3)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 40
hello:
.word\t0x1
.word\t0x2
.word\t0x3
.zero\t28";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_dword_arr() {
        let s = super::GenTool::gen_array::<u64>("hello", 10, &[(0, 1), (1, 2), (2, 3)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 80
hello:
.dword\t0x1
.dword\t0x2
.dword\t0x3
.zero\t56";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_float_arr() {
        let s = super::GenTool::gen_array::<f32>("hello", 10, &[(0, 1.0), (1, 2.0), (2, 3.0)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 40
hello:
.float\t1
.float\t2
.float\t3
.zero\t28";
        assert_eq!(s, raw_match);
    }
    #[test]
    fn test_gen_double_arr() {
        let s = super::GenTool::gen_array::<f64>("hello", 10, &[(0, 1.0), (1, 2.0), (2, 3.0)]);
        println!("{}", s);
        let raw_match = ".data
.align\t3
.globl\thello
.type\thello, @object
.size\thello, 80
hello:
.double\t1
.double\t2
.double\t3
.zero\t56";
        assert_eq!(s, raw_match);
    }
}
