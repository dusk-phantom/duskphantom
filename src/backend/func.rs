use super::block::*;
use std::collections::HashMap;
pub struct Func {
    name: String,
    args: Vec<String>,
    blocks: Vec<Block>,
    sorted_blocks: Vec<Block>,
    label_block_map: HashMap<String, Block>,
}
