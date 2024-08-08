use super::*;
/// 处理指令调度,将指令重新排序,以便于后续的优化
pub fn handle_inst_scheduling(func: &mut Func) -> Result<()> {
    for block in func.iter_bbs_mut() {
        let old_insts = block.insts();
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
    // let mut inst_id_map = HashMap::new();
    Ok(())
}

type InstID = usize;

/// 一个 bb 中只有一个 def, 即使是中端来的 phi, 在一个 bb 中也只有一个 def
type Defs = HashMap<WrapOperand, InstID>;
type Uses = HashMap<WrapOperand, HashSet<InstID>>;

/// FIXME 这实际上什么作用, 只是为了保证: 访存的相对顺序不会改变
type GlobalID = usize;

/// mem and reg
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum WrapOperand {
    Global(GlobalID),
    Stack(StackSlot),
    Reg(Reg),
}

struct WrapInst {
    id: InstID,
    inst: Inst,
}

/// 构造 defs, uses
fn construct_defs_uses(insts: &[Inst]) -> Result<(Vec<WrapInst>, Defs, Uses)> {
    /* ---------- 辅助宏 ---------- */
    macro_rules! insert_defs {
        ($inst: ident, $defs: ident, $id: ident) => {
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
        ($inst: ident, $uses: ident, $id: ident) => {
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
    let mut defs : Defs = HashMap::new();
    let mut uses : Uses = HashMap::new();
    let mut global_id : GlobalID = 0;
    for (id, inst) in insts.iter().enumerate() {
        // 添加 wrap_insts
        wrap_insts.insert(id, WrapInst {
            id,
            inst: inst.clone(),
        });  // id 就是 index, 尾插

        // 添加 defs 和 uses
        match inst {
            /* 算术指令, 注意一下, 这里面会有浮点, 注意 zero 不算是依赖 */
            Inst::Add(_) | Inst::Sub(_) | Inst::Sll(_) | Inst::Srl(_)
            | Inst::SRA(_) | Inst::Not(_) | Inst::And(_) | Inst::Or(_)
            | Inst::Xor(_) | Inst::Neg(_) | Inst::Slt(_) | Inst::Sltu(_)
            | Inst::Sgtu(_) | Inst::Seqz(_) | Inst::Snez(_) | Inst::Mv(_)
            /* 乘除法 */
            | Inst::Mul(_) | Inst::Div(_) | Inst::UDiv(_) | Inst::Rem(_) 
            /* 产生立即数 */
            | Inst::Li(_) | Inst::Lla(_) | Inst::LocalAddr(_)
            /* 浮点数比较 */
            | Inst::Feqs(_) | Inst::Fles(_) | Inst::Flts(_)
            /* convert */
            | Inst::I2f(_) | Inst::F2i(_) 
            /* 条件跳转 */
            | Inst::Beq(_) | Inst::Bne(_) | Inst::Blt(_) | Inst::Ble(_)
            | Inst::Bgt(_) | Inst::Bge(_) 
            /* 无条件跳转 */
            | Inst::Jmp(_) 
            /* use 参数列表, def A0 / FA0 */
            | Inst::Call(_) | Inst::Tail(_) | Inst::Ret 
            => {
                insert_defs!(inst, defs, id);
                insert_uses!(inst, uses, id);
            }, 
            Inst::Ld(_) | Inst::Lw(_) => {
                insert_defs!(inst, defs, id);
                let wrap = WrapOperand::Global(global_id);
                global_id += 1;
                uses.entry(wrap).or_default().insert(id);
            },
            Inst::Sd(_) | Inst::Sw(_) => {
                insert_uses!(inst, uses, id);
                let wrap = WrapOperand::Global(global_id);
                global_id += 1;
                defs.insert(wrap, id);
            },
            Inst::Load(ld) => {
                insert_defs!(inst, defs, id);
                let wrap = WrapOperand::Stack(*ld.src());
                uses.entry(wrap).or_default().insert(id);
            },
            Inst::Store(sd) => {
                let wrap = WrapOperand::Stack(*sd.dst());
                defs.insert(wrap, id);
                insert_uses!(inst, uses, id);
            },
        }
    }
    Ok((Vec::new(), defs, uses))
}
