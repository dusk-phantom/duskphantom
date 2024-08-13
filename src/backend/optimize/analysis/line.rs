use super::*;

impl Block {
    pub fn line(&self) -> usize {
        self.insts().len()
    }
}
impl Func {
    pub fn line(&self) -> usize {
        self.iter_bbs().map(|bb| bb.line()).sum()
    }
}
