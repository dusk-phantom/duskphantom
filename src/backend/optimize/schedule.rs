use super::*;
/// 处理指令调度,将指令重新排序,以便于后续的优化
pub fn handle_inst_scheduling(func: &mut Func) -> Result<()> {
    if CONFIG.num_parallel_for_block_gen_asm <= 1 {
        func.iter_bbs_mut()
            .try_for_each(handle_inst_scheduling_block)?;
    } else {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_block_gen_asm)
            .build()
            .with_context(|| context!())?;
        thread_pool.install(|| {
            func.iter_bbs_mut()
                .try_for_each(handle_inst_scheduling_block)
        })?;
    }
    Ok(())
}

fn handle_inst_scheduling_block(bb: &mut Block) -> Result<()> {
    let insts = bb.insts();
    let new_order = insts_scheduling(insts).with_context(|| context!())?;
    let new_insts = new_order
        .into_iter()
        .map(|id| insts[id].clone())
        .collect::<Vec<_>>();
    *bb.insts_mut() = new_insts;
    Ok(())
}

fn insts_scheduling(insts: &[Inst]) -> Result<Vec<InstID>> {
    let mut new_insts = Vec::new();
    let mut queue: Vec<StateOperand> = Vec::new();

    // 1. 构造依赖图
    let mut graph = Graph::new(insts).with_context(|| context!())?;

    while !graph.use_defs.is_empty() {
        // 1. 队列中所有的 cnt --
        for state in queue.iter_mut() {
            state.cnt -= 1;
            if state.cnt == 0 {
                graph.del_node(state.def).with_context(|| context!())?;
            }
        }

        // 2. 找到 cnt == 0 的指令, 从队列中删除, 并且删除依赖
        queue.retain(|state| state.cnt != 0);

        // 3. 搜集 indegree == 0 的节点, 还要排除已经在 queue 中的 node, 防止重复发射
        let remain_queue: Vec<InstID> = queue.iter().map(|state| state.def).collect();
        let mut no_deps = graph.collect_no_deps();
        no_deps.retain(|id| !remain_queue.contains(id));

        // 4. 选取两条指令
        let (inst1, inst2) = graph.select2_inst(&no_deps).with_context(|| context!())?;
        if let Some((state_operand, inst)) = inst1 {
            // 5. emit 两条指令
            new_insts.push(inst);
            // 6. 初始化状态并加入到队列中
            queue.push(state_operand);
        }
        if let Some((state_operand, inst)) = inst2 {
            // 5. emit 两条指令
            new_insts.push(inst);
            // 6. 初始化状态并加入到队列中
            queue.push(state_operand);
        }
    }

    Ok(new_insts)
}

/* ---------- ---------- 数据结构 ---------- ---------- */

type InstID = usize;

/// 看看 operand 准备的咋样了
#[derive(Debug)]
struct StateOperand {
    /// 定义的指令
    def: InstID,
    cnt: usize,
}

#[derive(Eq, PartialEq, Debug)]
enum InstType {
    Integer,
    Mul,
    DivRem,
    MemAccess,
    FloatPoint,
    /// 直接跳转/间接跳转
    Jmp,
}

impl Inst {
    fn character(&self) -> Result<(usize /* latency */, InstType)> {
        macro_rules! arithmetic_char {
            ($add:ident) => {
                if $add
                    .dst()
                    .reg()
                    .ok_or(anyhow!("arithmetic's dst is not reg"))?
                    .is_usual()
                {
                    Ok((1, InstType::Integer))
                } else {
                    Ok((4, InstType::FloatPoint))
                }
            };
        }
        match self {
            /* int or float */
            Inst::Add(add) => arithmetic_char!(add),
            Inst::Sub(sub) => arithmetic_char!(sub),
            Inst::Sll(sll) => arithmetic_char!(sll),
            Inst::Srl(srl) => arithmetic_char!(srl),
            Inst::SRA(sra) => arithmetic_char!(sra),
            Inst::Not(not_) => arithmetic_char!(not_),
            Inst::And(and_) => arithmetic_char!(and_),
            Inst::Or(or_) => arithmetic_char!(or_),
            Inst::Xor(xor) => arithmetic_char!(xor),
            Inst::Neg(neg) => arithmetic_char!(neg),
            Inst::Slt(slt) => arithmetic_char!(slt),
            Inst::Sltu(sltu) => arithmetic_char!(sltu),
            Inst::Sgtu(sgtu) => arithmetic_char!(sgtu),
            Inst::Seqz(seqz) => arithmetic_char!(seqz),
            Inst::Snez(snez) => arithmetic_char!(snez),
            Inst::Mv(mv) => arithmetic_char!(mv),
            /* int */
            Inst::LocalAddr(_) => Ok((1, InstType::Integer)),
            Inst::Li(_) | Inst::Lla(_) | Inst::Lui(_) => Ok((1, InstType::Integer)),
            /* mem access */
            Inst::F2i(_) | Inst::Fles(_) | Inst::Feqs(_) | Inst::Flts(_) | Inst::I2f(_) => {
                Ok((4, InstType::FloatPoint))
            }
            /* mem access */
            Inst::Ld(_)
            | Inst::Sd(_)
            | Inst::Lw(_)
            | Inst::Sw(_)
            | Inst::Load(_)
            | Inst::Store(_) => Ok((3, InstType::MemAccess)),
            /* jmp */
            Inst::Jmp(_)
            | Inst::Beq(_)
            | Inst::Bne(_)
            | Inst::Blt(_)
            | Inst::Ble(_)
            | Inst::Bgt(_)
            | Inst::Bge(_)
            | Inst::Call(_)
            | Inst::Ret
            | Inst::Tail(_) => Ok((1, InstType::Jmp)),
            /* div mul */
            Inst::Mul(_) => Ok((5, InstType::Mul)),
            Inst::Div(_) | Inst::UDiv(_) | Inst::Rem(_) => Ok((6, InstType::DivRem)),
        }
    }
}

#[derive(Debug)]
struct Graph<'a> {
    use_defs: HashMap<InstID /* use */, HashSet<InstID> /* def */>,
    def_uses: HashMap<InstID /* def */, HashSet<InstID> /* use */>,
    insts: &'a [Inst],
}

impl<'a> Graph<'a> {
    /// 选择两条指令出来
    #[allow(clippy::type_complexity)]
    fn select2_inst(
        &self,
        avail: &[InstID],
    ) -> Result<(
        Option<(StateOperand, InstID)>,
        Option<(StateOperand, InstID)>,
    )> {
        // 空的情况
        if avail.is_empty() {
            return Ok((None, None));
        }

        let mut ordered: Vec<(InstID, i32)> = Vec::new();
        for id in avail {
            let uses = self.use_defs.get(id).ok_or(anyhow!(""))?.len() as i32;
            ordered.push((*id, uses));
        }

        // 从大到小排序 (贪心)
        ordered.sort_by(|a, b| b.1.cmp(&a.1));

        for i in 0..ordered.len() {
            for j in i + 1..ordered.len() {
                let inst1_id = ordered[i].0;
                let inst2_id = ordered[j].0;
                let (latency1, type1) = self.insts[inst1_id]
                    .character()
                    .with_context(|| context!())?;
                let (latency2, type2) = self.insts[inst2_id]
                    .character()
                    .with_context(|| context!())?;
                if type1 != type2 || (type1 == type2 && type1 == InstType::Integer) {
                    return Ok((
                        Some((
                            StateOperand {
                                def: inst1_id,
                                cnt: latency1,
                            },
                            inst1_id,
                        )),
                        Some((
                            StateOperand {
                                def: inst2_id,
                                cnt: latency2,
                            },
                            inst2_id,
                        )),
                    ));
                }
            }
        }

        let inst1_id = ordered[0].0;
        let (latency1, _) = self.insts[inst1_id]
            .character()
            .with_context(|| context!())?;
        Ok((
            Some((
                StateOperand {
                    def: inst1_id,
                    cnt: latency1,
                },
                inst1_id,
            )),
            None,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum WrapOperand {
    /// sw/lw/sd/ld
    Global,
    Label(String),
    Stack(StackSlot),
    Reg(Reg),
}

impl<'a> Graph<'a> {
    fn new(insts: &'a [Inst]) -> Result<Self> {
        let bucket = Self::construct_bucket(insts).with_context(|| context!())?;

        // 初始化图
        let mut use_defs: HashMap<InstID, HashSet<InstID>> = HashMap::new();
        let mut def_uses: HashMap<InstID, HashSet<InstID>> = HashMap::new();
        for id in 0..insts.len() {
            use_defs.entry(id).or_default();
            def_uses.entry(id).or_default();
        }

        for (_, insts_flag) in bucket.iter() {
            // 几种情况, 滑动窗口, 建立依赖, win_l, win_r 是闭区间
            // r r r r r r r r r
            // w w w w w w w w w
            // r w r r w w r w r
            // r r r r w r r w r

            let mut win_l = usize::MAX; // dummy, 搞一个假的 def
            for win_r in 0..insts_flag.len() {
                if insts_flag[win_r].1 {
                    // is write
                    if win_l == usize::MAX {
                        for i in 0..win_r {
                            let def = insts_flag[i].0;
                            let use_ = insts_flag[win_r].0;
                            if def != use_ {
                                use_defs.entry(use_).or_default().insert(def);
                            }
                        }
                    } else {
                        for i in win_l..win_r {
                            let def = insts_flag[i].0;
                            let use_ = insts_flag[win_r].0;
                            if def != use_ {
                                use_defs.entry(use_).or_default().insert(def);
                            }
                        }
                    }
                    win_l = win_r;
                } else {
                    if win_l == usize::MAX {
                        continue;
                    }
                    // 出现了 read after write
                    let def = insts_flag[win_l].0;
                    let use_ = insts_flag[win_r].0;
                    if def != use_ {
                        use_defs.entry(use_).or_default().insert(def);
                    }
                }
            }
        }

        for id in (0..insts.len()).filter(|id| {
            matches!(
                insts[*id],
                Inst::Call(_)
                    | Inst::Beq(_)
                    | Inst::Bne(_)
                    | Inst::Bge(_)
                    | Inst::Bgt(_)
                    | Inst::Ble(_)
                    | Inst::Blt(_)
                    | Inst::Ret
                    | Inst::Jmp(_)
            )
        }) {
            // call 依赖于前面所有指令的指令
            for i in 0..id {
                use_defs.entry(id).or_default().insert(i);
            }
            // 后面所有指令依赖于这条 call
            for i in id + 1..insts.len() {
                use_defs.entry(i).or_default().insert(id);
            }
        }

        for (u, deps) in use_defs.iter() {
            for d in deps {
                def_uses.entry(*d).or_default().insert(*u);
            }
        }

        Ok(Self {
            use_defs,
            def_uses,
            insts,
        })
    }

    #[inline]
    fn collect_no_deps(&self) -> Vec<InstID> {
        let mut no_deps = Vec::new();
        for (id, deps) in self.use_defs.iter() {
            if deps.is_empty() {
                no_deps.push(*id);
            }
        }
        no_deps
    }

    fn del_node(&mut self, id: InstID) -> Result<()> {
        // id当且仅当它依赖的指令都执行完了, 才能被删除
        assert!(self.use_defs.get(&id).unwrap().is_empty());

        // dbg!("--- before ---");
        // dbg!(&self.def_uses);
        // dbg!(&self.use_defs);

        let use_insts = self.def_uses.remove(&id).unwrap_or_default();
        for use_inst in use_insts.iter() {
            if let Some(defs) = self.use_defs.get_mut(use_inst) {
                defs.remove(&id);
            }
        }
        self.use_defs.remove(&id).with_context(|| context!())?;

        // dbg!("--- after ---");
        // dbg!(&self.def_uses);
        // dbg!(&self.use_defs);

        Ok(())
    }
}

type IsW = bool;

impl<'a> Graph<'a> {
    #[allow(clippy::type_complexity)]
    fn construct_bucket(insts: &[Inst]) -> Result<HashMap<WrapOperand, Vec<(InstID, IsW)>>> {
        let mut bucket: HashMap<WrapOperand, Vec<(InstID, IsW)>> = HashMap::new();

        let mut reg_label: HashMap<Reg, String> = HashMap::new();

        for (id, inst) in insts.iter().enumerate() {
            match inst {
                /* 无条件跳转 */
                Inst::Beq(_)
                | Inst::Bne(_)
                | Inst::Blt(_)
                | Inst::Ble(_)
                | Inst::Bgt(_)
                | Inst::Bge(_)
                | Inst::Tail(_)
                | Inst::Ret
                | Inst::Jmp(_)
                | Inst::Call(_) => { /* 对于 bucket 啥也不干, 后面再单独处理 */ }
                Inst::Ld(ld) => {
                    let base = ld.base();
                    if let Some(label) = reg_label.get(base) {
                        bucket
                            .entry(WrapOperand::Label(label.clone()))
                            .or_default()
                            .push((id, false));
                    } else {
                        bucket
                            .entry(WrapOperand::Global)
                            .or_default()
                            .push((id, false));
                    }
                }
                Inst::Lw(lw) => {
                    let base = lw.base();
                    if let Some(label) = reg_label.get(base) {
                        bucket
                            .entry(WrapOperand::Label(label.clone()))
                            .or_default()
                            .push((id, false));
                    } else {
                        bucket
                            .entry(WrapOperand::Global)
                            .or_default()
                            .push((id, false));
                    }
                }
                Inst::Sd(sd) => {
                    let base = sd.base();
                    if let Some(label) = reg_label.get(base) {
                        bucket
                            .entry(WrapOperand::Label(label.clone()))
                            .or_default()
                            .push((id, true));
                    } else {
                        bucket
                            .entry(WrapOperand::Global)
                            .or_default()
                            .push((id, true));
                    }
                }
                Inst::Sw(sw) => {
                    let base = sw.base();
                    if let Some(label) = reg_label.get(base) {
                        bucket
                            .entry(WrapOperand::Label(label.clone()))
                            .or_default()
                            .push((id, true));
                    } else {
                        bucket
                            .entry(WrapOperand::Global)
                            .or_default()
                            .push((id, true));
                    }
                }
                Inst::Store(sd) => {
                    bucket
                        .entry(WrapOperand::Stack(*sd.dst()))
                        .or_default()
                        .push((id, true));
                }
                Inst::Load(ld) => {
                    bucket
                        .entry(WrapOperand::Stack(*ld.src()))
                        .or_default()
                        .push((id, false));
                }
                Inst::Lla(lla) => {
                    let reg = lla.dst();
                    let label = lla.label().to_string();
                    reg_label.insert(*reg, label);
                }
                _ => { /* 算术指令, 不用做特殊处理 */ }
            }
            for reg in inst.defs() {
                let reg = *reg;
                if reg == REG_ZERO {
                    continue;
                }
                bucket
                    .entry(WrapOperand::Reg(reg))
                    .or_default()
                    .push((id, true));
            }
            for reg in inst.uses() {
                let reg = *reg;
                if reg == REG_ZERO {
                    continue;
                }
                bucket
                    .entry(WrapOperand::Reg(reg))
                    .or_default()
                    .push((id, false));
            }
        }

        Ok(bucket)
    }
}

#[cfg(test)]
impl<'a> Graph<'a> {
    #[allow(dead_code)]
    pub fn gen_inst_dependency_graph_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");

        // gen node id
        for (id, inst) in self.insts.iter().enumerate() {
            let inst_str = inst.gen_asm();
            dot.push_str(&format!(
                "node{} [label=\"[{}]:  {}\"];\n",
                id, id, inst_str
            ));
        }

        for (use_, defs) in self.use_defs.iter() {
            for def in defs.iter() {
                dot.push_str(&format!("node{} -> node{};\n", use_, def));
            }
        }
        dot.push_str("}\n");
        dot
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn construct_func() -> Func {
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

    // #[test]
    // fn construct_graph_test() {
    //     let f = construct_func();
    //     let bb = f.entry();
    //     let insts = bb.insts()[0..bb.insts().len() - 1].to_vec();
    //     let graph = Graph::new(&insts).unwrap();
    // }

    // #[test]
    // fn test_gen_dot_graph_for_inst_dependency_graph() {
    //     let f = construct_func();
    //     let bb = f.entry();
    //     let insts = bb.insts();
    //     let graph = Graph::new(insts).unwrap();
    //     let dot = graph.gen_inst_dependency_graph_dot();
    //     println!("{}", dot);
    //     println!("{}", f.entry().gen_asm());
    //     dbg!(&graph.use_defs);
    // }

    #[test]
    fn debug_schedule() {
        let f = construct_func();
        let bb1 = f.entry();
        let _new_insts = {
            let insts = bb1.insts().clone();
            insts_scheduling(&insts).unwrap()
        };
    }
}
