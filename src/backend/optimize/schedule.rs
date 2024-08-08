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
    let mut new_insts = Vec::new();
    let mut queue: Vec<StateOperand> = Vec::new();

    // 1. 构造依赖图
    let mut graph = Graph::new(insts);
    // TODO while 循环, 进行指令调度
    while !graph.graph.is_empty() {
        // 1. 队列中所有的 cnt --
        for state in queue.iter_mut() {
            state.cnt -= 1;
        }
        // 2. 找到 cnt == 0 的指令, 从队列中删除, 并且删除依赖
        for i in (0..queue.len()) {
            if queue[i].cnt == 0 {
                let ready = &queue[i].op; // def -> ready
                let def_inst = *graph.defs.get(ready).ok_or(anyhow!("not found in defs"))?;
                graph.del_node(def_inst);
                queue.remove(i);
            }
        }

        // 3. 搜集 indegree == 0 的节点
        let no_deps = graph.collect_no_deps();
        // 4. 选取两条指令
        let (inst1, inst2) = graph.select2_inst(&no_deps);
        if let Some(inst1) = inst1 {
            // 5. emit 两条指令
            let inst1 = &graph.insts[inst1];
            new_insts.push(inst1.inst.clone());
            // 6. 初始化状态并加入到队列中
        }
        // 6. 加入队列中
    }

    Ok(new_insts)
}

/* ---------- ---------- 数据结构 ---------- ---------- */

/// 看看 operand 准备的咋样了
struct StateOperand {
    op: WrapOperand,
    cnt: usize,
}

type InstID = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum WrapOperand {
    /// lw, ld, sw, sd 会用这个, 保证相对顺序
    /// jmp, ret 会用这个, 保证在倒数第二条指令之后
    PreInst(InstID),
    Stack(StackSlot),
    Reg(Reg),
}

#[derive(Debug)]
struct WrapInst {
    id: InstID,
    inst: Inst,
}

enum InstType {
    Arithmetic,
    MulDivRem,
    MemAccess,
    FloatPoint,
    /// 直接跳转/间接跳转
    Jmp,
}

impl WrapInst {
    fn character(&self) -> Result<(usize /* latency */, InstType)> {
        macro_rules! arithmn_char {
            ($add:ident) => {
                if $add
                    .dst()
                    .reg()
                    .ok_or(anyhow!("arithmn's dst is not reg"))?
                    .is_usual()
                {
                    Ok((1, InstType::Arithmetic))
                } else {
                    Ok((4, InstType::FloatPoint))
                }
            };
        }
        match &self.inst {
            Inst::Add(add) => arithmn_char!(add),
            Inst::Sub(sub) => arithmn_char!(sub),
            Inst::Sll(sll) => arithmn_char!(sll),
            Inst::Srl(srl) => arithmn_char!(srl),
            Inst::SRA(sra) => arithmn_char!(sra),
            Inst::Not(not_) => arithmn_char!(not_),
            Inst::And(and_) => arithmn_char!(and_),
            Inst::Or(or_) => arithmn_char!(or_),
            Inst::Xor(xor) => arithmn_char!(xor),
            Inst::Neg(neg) => arithmn_char!(neg),
            Inst::Slt(slt) => arithmn_char!(slt),
            Inst::Sltu(sltu) => arithmn_char!(sltu),
            Inst::Sgtu(sgtu) => arithmn_char!(sgtu),
            Inst::Seqz(seqz) => arithmn_char!(seqz),
            Inst::Snez(snez) => arithmn_char!(snez),
            Inst::Mv(mv) => arithmn_char!(mv),
            Inst::Li(_) | Inst::Lla(_) => Ok((1, InstType::Arithmetic)),
            Inst::F2i(_) | Inst::Fles(_) | Inst::Feqs(_) | Inst::Flts(_) | Inst::I2f(_) => {
                Ok((4, InstType::FloatPoint))
            }
            Inst::Ld(_) => todo!(),
            Inst::Sd(_) => todo!(),
            Inst::Lw(_) => todo!(),
            Inst::Sw(_) => todo!(),
            Inst::Load(_) => todo!(),
            Inst::Store(_) => todo!(),
            Inst::LocalAddr(_) => todo!(),

            Inst::Jmp(_) => todo!(),
            Inst::Beq(_) => todo!(),
            Inst::Bne(_) => todo!(),
            Inst::Blt(_) => todo!(),
            Inst::Ble(_) => todo!(),
            Inst::Bgt(_) => todo!(),
            Inst::Bge(_) => todo!(),
            Inst::Call(_) => todo!(),
            Inst::Tail(_) => todo!(),
            Inst::Ret => todo!(),
            Inst::Mul(_) => todo!(),
            Inst::Div(_) => todo!(),
            Inst::UDiv(_) => todo!(),
            Inst::Rem(_) => todo!(),
        }
    }
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
    /// 选择两条指令出来
    fn select2_inst(&self, avail: &[InstID]) -> (Option<InstID>, Option<InstID>) {
        todo!()
    }
}

impl Graph {
    fn new(insts: &[Inst]) -> Self {
        let (insts, defs, uses) = Self::construct_defs_uses(insts);
        let mut graph: HashMap<InstID, HashSet<InstID>> = HashMap::new();
        for (operand, use_insts) in uses.iter() {
            if let WrapOperand::PreInst(pre_mem_inst_id) = operand {
                // sw/sd/lw/ld
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

    #[inline]
    fn del_node(&mut self, id: InstID) {
        self.graph.remove(&id);
        for deps in self.graph.values_mut() {
            deps.remove(&id);
        }
    }
}

impl Graph {
    #[allow(clippy::type_complexity)]
    fn construct_defs_uses(
        insts: &[Inst],
    ) -> (
        Vec<WrapInst>,
        HashMap<WrapOperand, InstID>,
        HashMap<WrapOperand, HashSet<InstID>>,
    ) {
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
        let mut pre_store: InstID = 0;

        for (id, inst) in insts.iter().enumerate() {
            // 添加 wrap_insts
            wrap_insts.insert(
                id,
                WrapInst {
                    id,
                    inst: inst.clone(),
                },
            ); // id 就是 index, 尾插

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
                    let wrap = WrapOperand::PreInst(pre_store);
                    uses.entry(wrap).or_default().insert(id);
                    /* ----- 不要忘了 use ----- */
                    insert_uses!(inst, uses, id);
                    pre_store = id;
                }
                Inst::Sd(_) | Inst::Sw(_) => {
                    insert_uses!(inst, uses, id);
                    // raw
                    defs.insert(WrapOperand::PreInst(id), id); // sw 也要保证顺序
                    // waw
                    uses.entry(WrapOperand::PreInst(pre_store)).or_default().insert(id);
                    pre_store = id;
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
