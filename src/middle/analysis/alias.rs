use std::collections::HashMap;

use crate::{
    backend::from_self::downcast_ref,
    middle::ir::{
        instruction::{memory_op_inst::GetElementPtr, InstType},
        Operand, ValueType,
    },
};

/// Check if two operands (maybe with GEP) can alias.
pub fn can_alias(a: Operand, b: Operand) -> bool {
    let (ptr_a, offset_a) = split_gep(a);
    let (ptr_b, offset_b) = split_gep(b);
    can_ptr_alias(ptr_a, ptr_b) && can_offset_overlap(offset_a, offset_b)
}

/// Check if two operands (without GEP) can alias.
fn can_ptr_alias(a: Operand, b: Operand) -> bool {
    // If any of them is param, they can alias
    if let Operand::Parameter(_) = a {
        return true;
    }
    if let Operand::Parameter(_) = b {
        return true;
    }

    // Global variable alias only when they're the same
    if let Operand::Global(a) = a {
        if let Operand::Global(b) = b {
            return a == b;
        }
    }

    // Alloc instruction alias only when they're the same
    if let Operand::Instruction(a) = a {
        if let Operand::Instruction(b) = b {
            return a == b;
        }
    }

    // Operand of different type will not alias
    false
}

/// Check if two sets of GEP offsets can overlap.
fn can_offset_overlap(a: HashMap<ValueType, Operand>, b: HashMap<ValueType, Operand>) -> bool {
    for (key, a_op) in a.iter() {
        if let Some(b_op) = b.get(key) {
            if !can_equal(a_op.clone(), b_op.clone()) {
                return false;
            }
        }
    }
    true
}

/// Split GEP instruction into base pointer and offset.
fn split_gep(op: Operand) -> (Operand, HashMap<ValueType, Operand>) {
    let mut base = op;
    let mut offset = HashMap::new();
    while let Operand::Instruction(inst) = base {
        if inst.get_type() != InstType::GetElementPtr {
            break;
        }

        // Update base
        base = inst.get_operand().first().unwrap().clone();

        // Update offset
        let gep = downcast_ref::<GetElementPtr>(inst.as_ref().as_ref());
        let mut element_type = gep.element_type.clone();
        for op in inst.get_operand().iter().skip(1) {
            offset.insert(element_type.clone(), op.clone());
            element_type = element_type.get_sub_type().unwrap().clone();
        }
    }
    (base, offset)
}

/// Check if two indexing operands can equal.
fn can_equal(a: Operand, b: Operand) -> bool {
    match (a, b) {
        // Constants only equal when they're the same
        (Operand::Constant(a), Operand::Constant(b)) => a == b,

        // Other operands can always equal (give up predicate analysis)
        _ => true,
    }
}
