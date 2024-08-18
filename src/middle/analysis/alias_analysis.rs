use std::collections::{HashMap, HashSet};

use crate::{
    backend::from_self::downcast_ref,
    middle::ir::{
        instruction::{memory_op_inst::GetElementPtr, InstType},
        Constant, Operand, ValueType,
    },
};

#[derive(Clone)]
pub enum EffectRange {
    All,
    Some(HashSet<Operand>),
}

/// Check if two effect range must be the same.
impl PartialEq for EffectRange {
    fn eq(&self, another: &Self) -> bool {
        match (self, another) {
            (EffectRange::Some(a), EffectRange::Some(b)) => {
                if a.len() != 1 || b.len() != 1 {
                    return false;
                }
                // TODO judge equal with GVN
                a.iter().next().unwrap() == b.iter().next().unwrap()
            }
            _ => false,
        }
    }
}

impl Default for EffectRange {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRange {
    /// Create an empty effect range.
    pub fn new() -> Self {
        EffectRange::Some(HashSet::new())
    }

    /// Check if two effect ranges conflict when parallelized.
    pub fn can_conflict(&self, another: &EffectRange) -> bool {
        match (self, another) {
            (EffectRange::All, EffectRange::All) => true,
            (EffectRange::All, EffectRange::Some(_)) => true,
            (EffectRange::Some(_), EffectRange::All) => true,
            (EffectRange::Some(a), EffectRange::Some(b)) => a
                .iter()
                .any(|a_op| b.iter().any(|b_op| can_op_conflict(a_op, b_op))),
        }
    }

    /// Check if two effect ranges can alias.
    pub fn can_alias(&self, another: &EffectRange) -> bool {
        match (self, another) {
            (EffectRange::All, EffectRange::All) => true,
            (EffectRange::All, EffectRange::Some(_)) => true,
            (EffectRange::Some(_), EffectRange::All) => true,
            (EffectRange::Some(a), EffectRange::Some(b)) => a
                .iter()
                .any(|a_op| b.iter().any(|b_op| can_op_alias(a_op, b_op))),
        }
    }

    /// Merge two effect ranges.
    pub fn merge(&mut self, another: &EffectRange) {
        if let EffectRange::All = another {
            *self = EffectRange::All;
        } else if let (EffectRange::Some(a), EffectRange::Some(b)) = (self, another) {
            a.extend(b.iter().cloned());
        }
    }

    /// Check if the effect range is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            EffectRange::All => false,
            EffectRange::Some(set) => set.is_empty(),
        }
    }

    /// Get the only operand if the effect range contains only one operand.
    pub fn get_single(&self) -> Option<&Operand> {
        match self {
            EffectRange::All => None,
            EffectRange::Some(set) => {
                if set.len() == 1 {
                    set.iter().next()
                } else {
                    None
                }
            }
        }
    }

    /// Dump effect range to string.
    pub fn dump(&self) -> String {
        match self {
            EffectRange::All => "all".to_string(),
            EffectRange::Some(set) => {
                let mut set: Vec<_> = set.iter().map(Operand::to_string).collect();
                set.sort();
                set.join(", ")
            }
        }
    }
}

impl From<Operand> for EffectRange {
    fn from(op: Operand) -> Self {
        EffectRange::Some([op].into_iter().collect())
    }
}

impl From<HashSet<Operand>> for EffectRange {
    fn from(set: HashSet<Operand>) -> Self {
        EffectRange::Some(set)
    }
}

/// Check if two operands (maybe with GEP) can alias.
fn can_op_alias(a: &Operand, b: &Operand) -> bool {
    let (ptr_a, offset_a) = split_gep(a);
    let (ptr_b, offset_b) = split_gep(b);
    can_ptr_alias(&ptr_a, &ptr_b) && can_offset_overlap(offset_a, offset_b)
}

/// Check if two operands cans conflict when parallelized.
fn can_op_conflict(a: &Operand, b: &Operand) -> bool {
    a != b && can_op_alias(a, b)
}

/// Check if two operands (without GEP) can alias.
fn can_ptr_alias(a: &Operand, b: &Operand) -> bool {
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
fn split_gep(op: &Operand) -> (Operand, HashMap<ValueType, Operand>) {
    let mut base = op.clone();
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
            if let Some(old_offset) = offset.get_mut(&element_type) {
                // Handle only +0 or 0+ in this case, UB otherwise
                if let Operand::Constant(Constant::Int(0)) = old_offset {
                    *old_offset = op.clone();
                } else if let Operand::Constant(Constant::Int(0)) = op {
                    // Do nothing
                } else {
                    unimplemented!("Unsupported GEP offset: {} + {}", old_offset, op);
                }
            } else {
                offset.insert(element_type.clone(), op.clone());
            }
            if let Some(subtype) = element_type.get_sub_type() {
                element_type = subtype.clone();
            }
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
