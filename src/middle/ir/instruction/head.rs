use super::*;
pub struct Head {
    inst_type: InstType,
}

impl Head {
    pub fn new() -> Self {
        Self {
            inst_type: InstType::Head,
        }
    }
}
