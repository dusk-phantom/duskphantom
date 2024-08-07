use super::*;
use anyhow::Result;
use std::collections::HashMap;

use crate::middle;

macro_rules! branch {
    ($beq:ident /* beq */,  $label_map:ident /* label_map */) => {{
        *$beq.label_mut() = $label_map
            .get(&**$beq.label())
            .ok_or(anyhow!("not found label"))?
            .clone()
            .into();
    }};
}

impl IRBuilder {
    // pub fn label_rename_funcs(backend: &mut [Func], middle: &[FunPtr]) -> Result<()> {
    //     // 这里 zip 一起遍历, 是因为: 构造的时候就是一一对应的
    //     for (back, mid) in backend.iter_mut().zip(middle.iter()) {
    //         Self::label_rename_func(back, mid)?;
    //     }
    //     Ok(())
    // }

    pub fn label_rename_func(backend: &mut Func, middle: &middle::ir::Function) -> Result<()> {
        let label_map: HashMap<String, String> = middle
            .bfs_iter()
            .map(|bb| {
                let key = Self::label_name_from(&bb);
                let val = format!(".L{}_{}", middle.name.clone(), bb.name.clone());
                (key, val)
            })
            .collect();
        for bb in backend.iter_bbs_mut() {
            // 改 bb 的 label
            let new_label = label_map
                .get(bb.label())
                .ok_or(anyhow!("label not found"))?
                .clone();
            bb.set_label(&new_label);
            for inst in bb.insts_mut() {
                match inst {
                    Inst::Jmp(jmp) => {
                        *jmp.dst_mut() = label_map
                            .get(&*jmp.dst().label().ok_or(anyhow!("get label failed"))?)
                            .ok_or(anyhow!("not found label"))?
                            .clone()
                            .into();
                    }
                    Inst::Beq(beq) => {
                        branch!(beq, label_map);
                    }
                    Inst::Bne(bne) => {
                        branch!(bne, label_map);
                    }
                    Inst::Blt(blt) => {
                        branch!(blt, label_map);
                    }
                    Inst::Ble(ble) => {
                        branch!(ble, label_map);
                    }
                    Inst::Bgt(bgt) => {
                        branch!(bgt, label_map);
                    }
                    Inst::Bge(bge) => {
                        branch!(bge, label_map);
                    }
                    _ => { /* do nothing */ }
                }
            }
        }
        // 改 jmp / branch 的 label
        Ok(())
    }
}
