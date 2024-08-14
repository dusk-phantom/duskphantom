#![no_main]

use libfuzzer_sys::fuzz_target;
use reg_set_fuzz::*;
use std::vec::Vec;

fuzz_target!(|actions:Vec<Action>| {
    use std::collections::HashMap;
    let  mut rgs=HashMap::new();
    apply_actions(&mut rgs, actions);
});
