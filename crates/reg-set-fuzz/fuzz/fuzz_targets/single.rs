#![no_main]

use libfuzzer_sys::fuzz_target;
use reg_set_fuzz::*;
use std::vec::Vec;
use std::collections::HashSet;


fuzz_target!(|actions:Vec<SingleAction>| {
    use reg_set_fuzz::*;
    let  mut rg=RegSet::new();
    let mut rge=HashSet::new();
    apply_single_actions(&mut rg,&mut rge, actions);
});