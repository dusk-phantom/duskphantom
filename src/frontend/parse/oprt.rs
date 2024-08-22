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

use super::*;

pub fn unary_op(input: &mut &str) -> PResult<UnaryOp> {
    dispatch! { peek(any);
        '!' => token("!").value(UnaryOp::Not),
        '-' => token("-").value(UnaryOp::Neg),
        '+' => token("+").value(UnaryOp::Pos),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 0 operators, left to right
pub fn binary_op_lv0(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '*' => token("*").value(BinaryOp::Mul),
        '/' => token("/").value(BinaryOp::Div),
        '%' => token("%").value(BinaryOp::Mod),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 1 operators, left to right
pub fn binary_op_lv1(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token("+").value(BinaryOp::Add),
        token("-").value(BinaryOp::Sub),
    ))
    .parse_next(input)
}

/// Level 2 operators, left to right
pub fn binary_op_lv2(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token(">>").value(BinaryOp::Shr),
        token("<<").value(BinaryOp::Shl),
    ))
    .parse_next(input)
}

/// Level 3 operators, left to right
pub fn binary_op_lv3(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '>' => alt((
            token(">=").value(BinaryOp::Ge),
            token(">").value(BinaryOp::Gt),
        )),
        '<' => alt((
            token("<=").value(BinaryOp::Le),
            token("<").value(BinaryOp::Lt),
        )),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 4 operators, left to right
pub fn binary_op_lv4(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token("==").value(BinaryOp::Eq),
        token("!=").value(BinaryOp::Ne),
    ))
    .parse_next(input)
}

/// Level 5 operators, left to right
pub fn binary_op_lv5(input: &mut &str) -> PResult<BinaryOp> {
    token("&").value(BinaryOp::BitAnd).parse_next(input)
}

/// Level 6 operators, left to right
pub fn binary_op_lv6(input: &mut &str) -> PResult<BinaryOp> {
    token("^").value(BinaryOp::BitXor).parse_next(input)
}

/// Level 7 operators, left to right
pub fn binary_op_lv7(input: &mut &str) -> PResult<BinaryOp> {
    token("|").value(BinaryOp::BitOr).parse_next(input)
}

/// Level 8 operators, left to right
pub fn binary_op_lv8(input: &mut &str) -> PResult<BinaryOp> {
    token("&&").value(BinaryOp::And).parse_next(input)
}

/// Level 9 operators, left to right
pub fn binary_op_lv9(input: &mut &str) -> PResult<BinaryOp> {
    token("||").value(BinaryOp::Or).parse_next(input)
}
