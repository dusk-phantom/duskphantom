use super::block::*;
#[allow(unused)]
pub struct Func {
    name: String,
    args: Vec<String>,
    // bacic blocks
    bbs: Vec<Block>,
    // sorted basic blocks by dict order of label,ascendingly
    sorted_bbs: Vec<Block>,
}
