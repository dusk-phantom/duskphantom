use super::*;

impl_conversion_inst!(I2fInst, "fcvt.s.w");
impl_conversion_inst!(F2iInst, "fcvt.w.s", "rtz");

// impl conversion to Inst
impl_inst_convert!(I2fInst, I2f);
impl_inst_convert!(F2iInst, F2i);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_i2f_inst() {
        let inst = I2fInst::new(REG_FA0.into(), REG_A0.into());
        assert_eq!(inst.gen_asm(), "fcvt.s.w fa0,a0");
    }
    #[test]
    fn test_f2i_inst() {
        let inst = F2iInst::new(REG_A0.into(), REG_FA0.into());
        assert_eq!(inst.gen_asm(), "fcvt.w.s a0,fa0,rtz");
    }
}
