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

use rayon::iter::Either;

use super::*;

pub fn box_stmt(input: &mut &str) -> PResult<Box<Stmt>> {
    stmt.map(Box::new).parse_next(input)
}

pub fn vec_stmt(input: &mut &str) -> PResult<Vec<Stmt>> {
    repeat(0.., stmt).parse_next(input)
}

/// Expression with semicolon.
pub fn expr_sc(input: &mut &str) -> PResult<Expr> {
    (expr, cut_err(token(";")))
        .map(|(e, _)| e)
        .parse_next(input)
}

/// Decl or Expr.
pub fn decl_or_expr(input: &mut &str) -> PResult<Either<Decl, Expr>> {
    alt((decl.map(Either::Left), expr_sc.map(Either::Right))).parse_next(input)
}

pub fn stmt(input: &mut &str) -> PResult<Stmt> {
    let disp = dispatch! { peek(any);
        'b' => (token("break"), cut_err(token(";"))).value(Stmt::Break),
        'c' => (token("continue"), cut_err(token(";"))).value(Stmt::Continue),
        'i' => (token("if"), cut_err((paren(expr), box_stmt, opt((token("else"), box_stmt)))))
            .map(|(_, (cond, pass, fail))| Stmt::If(cond, pass, fail.map_or(Stmt::Block(vec![]).into(), |(_, s)| s))),
        'w' => (token("while"), cut_err((paren(expr), box_stmt))).map(|(_, (cond, body))| Stmt::While(cond, body)),
        'd' => (token("do"), cut_err((box_stmt, token("while"), paren(expr), token(";"))))
            .map(|(_, (body, _, cond, _))| Stmt::DoWhile(body, cond)),
        'r' => (token("return"), cut_err((opt(expr), token(";"))))
            .map(|(_, (e, _))| Stmt::Return(e)),
        '{' => curly(cut_err(vec_stmt)).map(Stmt::Block),
        _ => fail
    };
    alt((
        disp,
        decl.map(Stmt::Decl),
        (opt(terminated(expr, token("="))), expr_sc).map(|(lval, expr)| Stmt::Expr(lval, expr)),
        token(";").value(Stmt::Nothing),
    ))
    .parse_next(input)
}
