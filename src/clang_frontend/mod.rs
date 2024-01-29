use std::process::Command;
use tempfile::NamedTempFile;

pub struct Program {}

impl Program {
    pub fn parse(file: &str) -> Self {
        //创建临时文件
        let tmp_file = NamedTempFile::new().expect("msg: create tmp file failed");

        let cmd = Command::new("clang")
            .arg("-S")
            .arg("-emit-llvm")
            .arg(file)
            .arg("-o")
            //指定输出到临时文件中
            .output()
            .expect("msg: clang failed");

        Self {}
    }
}
