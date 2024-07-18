#[allow(unused)]
use super::*;

#[cfg(test)]
mod test_reg_def_use {
    use super::*;
    #[test]
    fn test_seqz() {
        let mut reg_gener = RegGenerator::new();
        let dst = reg_gener.gen_virtual_usual_reg();
        let src = reg_gener.gen_virtual_usual_reg();
        let seqz = SeqzInst::new(dst.into(), src.into());
        assert_eq!(seqz.uses(), vec![&src]);
        assert_eq!(seqz.defs(), vec![&dst]);
    }
    #[test]
    fn test_repeat() {
        let mul = MulInst::new(REG_A0.into(), REG_A0.into(), REG_A0.into());
        assert_eq!(mul.uses(), vec![&REG_A0]);
        assert_eq!(mul.defs(), vec![&REG_A0]);
        let beq = BeqInst::new(REG_A0, REG_A0, "a".into());
        assert_eq!(beq.uses(), vec![&REG_A0]);
        let beq2 = BeqInst::new(REG_A0, REG_A1, "a".into());
        assert_eq!(beq2.uses(), vec![&REG_A0, &REG_A1]);
    }
    #[test]
    fn test_mv() {
        let mv = MvInst::new(REG_A0.into(), REG_A1.into());
        assert_eq!(mv.uses(), vec![&REG_A1]);
        assert_eq!(mv.defs(), vec![&REG_A0]);
    }
}

#[cfg(test)]
mod test_reg_replace {}
