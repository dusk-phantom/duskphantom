// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
pub mod test_gep {
    // use compiler::backend::Operand;

    #[test]
    fn gep_example() {
        // let mut ir_builder = IRBuilder::new();
        // let ir_builder = Box::pin(ir_builder);
        // let mut module = Module::new(ObjPtr::new(&ir_builder));

        // let mut func = ir_builder.new_function("test_gep".to_owned(), ValueType::Void);
        // module.functions.push(func);

        // let mut bb = ir_builder.new_basicblock("test_gep_bb".to_owned());
        // func.entry = Some(bb);
        // func.exit = Some(bb);

        // bb.push_back(ir_builder.get_ret(None));
        // let tail = bb.get_last_inst();

        // let a_type = ValueType::Array(
        //     ValueType::Array(ValueType::Array(ValueType::Int, 10), 10),
        //     10,
        // );

        // // int a[10][10][10];
        // let a = ir_builder.get_alloca(
        //     ValueType::Array(ValueType::Array(ValueType::Int, 10), 10),
        //     10,
        // );

        // // a[1][2][3] = 4;
        // let gep = ir_builder.get_getelementptr(
        //     a_type,
        //     a,
        //     vec![
        //         Operand::Constant(0),
        //         Operand::Constant(1),
        //         Operand::Constant(2),
        //         Operand::Constant(3),
        //     ],
        // );

        // let st = ir_builder.get_store(operand::Operand::Constant(4), gep);

        // tail.insert_before(a);
        // tail.insert_before(gep);
        // tail.insert_before(st);
        // assert!(true);
    }
}
