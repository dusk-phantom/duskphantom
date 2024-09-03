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

/// A declaration.
/// Example: `int x = 4;`
#[derive(Clone, PartialEq, Debug)]
pub enum Decl {
    /// A declaration of a constant, optionally with assignment.
    /// Example:
    /// `const int x;` is `Const(Int, x, None)`
    /// `const int x = 4;` is `Const(Int, x, Some(Int(4)))`
    Const(Type, String, Option<Expr>),

    /// A declaration of a variable, optionally with assignment.
    /// Example:
    /// `int x;` is `Var(Int, x, None)`
    /// `int x = 4;` is `Var(Int, x, Some(Int(4)))`
    Var(Type, String, Option<Expr>),

    /// Stacked declarations.
    /// Example:
    /// `int x = 1, y = 2;` is `Stack([Var(Int, x, Some(Int(1))), Var(Int, y, Some(Int(2)))])`
    Stack(Vec<Decl>),

    /// A declaration of a function, optionally with implementation.
    /// Example:
    /// `void f(int x)` is `Func(Void, "f", [(Int, (Some("x"))], None)`
    /// `void f() { ... }` is `Func(Void, "f", [], Some(...))`
    Func(Type, String, Option<Box<Stmt>>),
}
