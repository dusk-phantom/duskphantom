use super::*;
/// 处理指令调度,将指令重新排序,以便于后续的优化
pub fn handle_inst_scheduling(func: &mut Func) -> Result<()> {
    for block in func.iter_bbs_mut() {
        let old_insts = block.insts();
        println!("{}", block.gen_asm());
        let new_insts = handle_block_scheduling(old_insts).with_context(|| context!())?;
        *block.insts_mut() = new_insts;
    }
    Ok(())
}

fn handle_block_scheduling(insts: &[Inst]) -> Result<Vec<Inst>> {
    // TODO 构造指令之间的依赖图
    construct_dependence_graph(insts).with_context(|| context!())?;
    // TODO while 循环, 进行指令调度
    Ok(insts.to_vec())
}

fn construct_dependence_graph(insts: &[Inst]) -> Result<()> {
    // 1. 为指令分配 id 并且建立: operand 与 id 的反向映射
    Ok(())
}

/* ---------- ---------- 数据结构 ---------- ---------- */

type InstID = usize;

/// mem and reg
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum WrapOperand {
    /// lw, ld, sw, sd 会用这个, 保证相对顺序
    /// jmp, ret 会用这个, 保证在倒数第二条指令之后
    PreInst(InstID),
    Stack(StackSlot),
    Reg(Reg),
}

// impl WrapOperand {
//     fn wrap_operand(op: &Operand) -> Result<Self> {
//         match op {
//             Operand::Reg(_) => todo!(),
//             Operand::Imm(_) => todo!(),
//             Operand::Fmm(_) => todo!(),
//             Operand::StackSlot(_) => todo!(),
//             Operand::Label(_) => todo!(),
//         }
//     }
// }

#[derive(Debug)]
struct WrapInst {
    id: InstID,
    inst: Inst,
}

#[derive(Debug)]
struct Graph {
    /// 依赖图, 指向所依赖的指令
    graph: HashMap<InstID /* use */, HashSet<InstID> /* def */>,
    /// 一个 bb 中只有一个 def, 即使是中端来的 phi, 在一个 bb 中也只有一个 def
    defs: HashMap<WrapOperand, InstID>,
    /// 注意, 这个 uses 会出现 : 来自其他 bb 的寄存器
    uses: HashMap<WrapOperand, HashSet<InstID>>,
    /// <id, WrapInst>
    insts: Vec<WrapInst>,
}

impl Graph {
    fn new(insts: &[Inst]) -> Self {
        let (insts, defs, uses) = Self::construct_defs_uses(insts);
        let mut graph: HashMap<InstID, HashSet<InstID>> = HashMap::new();
        for (operand, use_insts) in uses.iter() {
            if let WrapOperand::PreInst(pre_mem_inst_id) = operand {
                // sw/sd/lw/ld
                if *pre_mem_inst_id == usize::MAX {
                    continue; // 说明是当前基本块的第一个 mem access
                }
                for use_id in use_insts {
                    if use_id != pre_mem_inst_id {
                        graph.entry(*use_id).or_default().insert(*pre_mem_inst_id);
                    }
                }
            } else if let Some(def_inst) = defs.get(operand) {
                for use_id in use_insts {
                    if use_id != def_inst {
                        graph.entry(*use_id).or_default().insert(*def_inst);
                    }
                }
            }
        }
        // 还剩下一些指令, 比方说 lla/li 只有 def 没有 use
        for inst in insts.iter() {
            graph.entry(inst.id).or_default();
        }
        Self {
            graph,
            defs,
            uses,
            insts,
        }
    }
}

impl Graph {
    #[allow(clippy::type_complexity)]
    fn construct_defs_uses(
        insts: &[Inst]
    ) -> (Vec<WrapInst>, HashMap<WrapOperand, InstID>, HashMap<WrapOperand, HashSet<InstID>>) {
        /* ---------- 辅助宏 ---------- */
        macro_rules! insert_defs {
            ($inst:ident, $defs:ident, $id:ident) => {
                for _d in $inst.defs() {
                    if (_d.eq(&REG_ZERO)) {
                        continue;
                    }
                    let _wrap = WrapOperand::Reg(*_d);
                    $defs.insert(_wrap, $id);
                }
            };
        }

        macro_rules! insert_uses {
            ($inst:ident, $uses:ident, $id:ident) => {
                for _u in $inst.uses() {
                    if (_u.eq(&REG_ZERO)) {
                        continue;
                    }
                    let _wrap = WrapOperand::Reg(*_u);
                    $uses.entry(_wrap).or_default().insert($id);
                }
            };
        }

        /* ---------- 函数正文 ---------- */

        let mut wrap_insts = Vec::new();
        let mut defs: HashMap<WrapOperand, InstID> = HashMap::new();
        let mut uses: HashMap<WrapOperand, HashSet<InstID>> = HashMap::new();

        // 上一条 sw/lw/sd/ld 指令的 id
        let mut pre_mem_inst: InstID = usize::MAX;

        for (id, inst) in insts.iter().enumerate() {
            // 添加 wrap_insts
            wrap_insts.insert(id, WrapInst {
                id,
                inst: inst.clone(),
            }); // id 就是 index, 尾插

            // 添加 defs 和 uses
            match inst {
                /* 算术指令, 注意一下, 这里面会有浮点, 注意 zero 不算是依赖 */
                | Inst::Add(_)
                | Inst::Sub(_)
                | Inst::Sll(_)
                | Inst::Srl(_)
                | Inst::SRA(_)
                | Inst::Not(_)
                | Inst::And(_)
                | Inst::Or(_)
                | Inst::Xor(_)
                | Inst::Neg(_)
                | Inst::Slt(_)
                | Inst::Sltu(_)
                | Inst::Sgtu(_)
                | Inst::Seqz(_)
                | Inst::Snez(_)
                | Inst::Mv(_)
                /* 乘除法 */
                | Inst::Mul(_)
                | Inst::Div(_)
                | Inst::UDiv(_)
                | Inst::Rem(_)
                /* 产生立即数 */
                | Inst::Li(_)
                | Inst::Lla(_)
                | Inst::LocalAddr(_)
                /* 浮点数比较 */
                | Inst::Feqs(_)
                | Inst::Fles(_)
                | Inst::Flts(_)
                /* convert */
                | Inst::I2f(_)
                | Inst::F2i(_)
                /* use 参数列表, def A0 / FA0 */
                | Inst::Call(_) => {
                    insert_defs!(inst, defs, id);
                    insert_uses!(inst, uses, id);
                }
                /* 无条件跳转 */
                Inst::Tail(_) | Inst::Ret | Inst::Jmp(_) => {
                    // 最后的跳转, 依赖于前面所有指令执行完
                    for pre in 0..id {
                        let wrap = WrapOperand::PreInst(pre);
                        uses.entry(wrap).or_default().insert(id);
                    }
                }
                /* 条件跳转 */
                | Inst::Beq(_)
                | Inst::Bne(_)
                | Inst::Blt(_)
                | Inst::Ble(_)
                | Inst::Bgt(_)
                | Inst::Bge(_) => {
                    insert_uses!(inst, uses, id);
                    for pre in 0..id {
                        let wrap = WrapOperand::PreInst(pre);
                        uses.entry(wrap).or_default().insert(id);
                    }
                }
                Inst::Ld(_) | Inst::Lw(_) => {
                    insert_defs!(inst, defs, id);
                    /* ----- 这是为了确保 mem access 指令顺序一致 ----- */
                    let wrap = WrapOperand::PreInst(pre_mem_inst);
                    uses.entry(wrap).or_default().insert(id);
                    /* ----- 不要忘了 use ----- */
                    insert_uses!(inst, uses, id);
                    pre_mem_inst = id;
                }
                Inst::Sd(_) | Inst::Sw(_) => {
                    insert_uses!(inst, uses, id);
                    /* ----- 这是为了确保 mem access 指令顺序一致 ----- */
                    let wrap = WrapOperand::PreInst(pre_mem_inst);
                    uses.entry(wrap).or_default().insert(id); // sw 也要保证顺序
                    pre_mem_inst = id;
                }
                Inst::Load(ld) => {
                    insert_defs!(inst, defs, id);
                    let wrap = WrapOperand::Stack(*ld.src());
                    uses.entry(wrap).or_default().insert(id);
                }
                Inst::Store(sd) => {
                    let wrap = WrapOperand::Stack(*sd.dst());
                    defs.insert(wrap, id);
                    insert_uses!(inst, uses, id);
                }
            }
        }
        (wrap_insts, defs, uses)
    }

    #[inline]
    fn collect_no_deps(&self) -> Vec<InstID> {
        let mut no_deps = Vec::new();
        for (id, deps) in self.graph.iter() {
            if deps.is_empty() {
                no_deps.push(*id);
            }
        }
        no_deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consturct_func() -> Func {
        let mut entry = Block::new("entry".to_string());
        let ssa = StackAllocator::new();
        let mut rg = RegGenerator::new();

        // lla x33, sum
        let x32 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // lw x32, 0(x33)
        let lw = LwInst::new(x32, (0).into(), addr);
        entry.push_inst(lw.into());

        // lla x35, a
        let x34 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "a".into());
        entry.push_inst(lla.into());

        // lw x34, 0(x35)
        let lw = LwInst::new(x34, (0).into(), addr);
        entry.push_inst(lw.into());

        // addw x36, x34, x32
        let x36 = rg.gen_virtual_usual_reg();
        let add = AddInst::new(x36.into(), x34.into(), x32.into());
        entry.push_inst(add.into());

        // lla x37, sum
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // sw x36, 0(x37)
        let sw = SwInst::new(x36, (0).into(), addr);
        entry.push_inst(sw.into());

        // call getA
        let mut call = CallInst::new("getA".into());
        call.add_def(REG_A0);
        entry.push_inst(call.into());

        // mv x38, a0
        let x38 = rg.gen_virtual_usual_reg();
        let mv = MvInst::new(x38.into(), REG_A0.into());
        entry.push_inst(mv.into());

        // lla x40, sum
        let x39 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // lw x39, 0(x40)
        let lw = LwInst::new(x39, (0).into(), addr);
        entry.push_inst(lw.into());

        // lla x42, a
        let x41 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "a".into());
        entry.push_inst(lla.into());

        // lw x41, 0(x42)
        let lw = LwInst::new(x41, (0).into(), addr);
        entry.push_inst(lw.into());

        // addw x43, x41, x39
        let x43 = rg.gen_virtual_usual_reg();
        let add = AddInst::new(x43.into(), x41.into(), x39.into());
        entry.push_inst(add.into());

        // lla x44, sum
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // sw x43, 0(x44)
        let sw = SwInst::new(x43, (0).into(), addr);
        entry.push_inst(sw.into());

        entry.push_inst(Inst::Ret);

        let mut f = Func::new("f2".to_string(), vec![], entry);
        f.stack_allocator_mut().replace(ssa);
        f.reg_gener_mut().replace(rg);
        f
    }

    #[test]
    fn construct_graph_test() {
        let f = consturct_func();
        let bb = f.entry();
        let insts = bb.insts();
        let graph = Graph::new(insts);
        dbg!(&graph.graph);
        println!("{}", f.gen_asm());
        dbg!(&graph.collect_no_deps());
    }
}
