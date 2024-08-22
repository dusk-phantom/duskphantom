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

mod decl;
mod expr;
#[macro_use]
mod macros;
mod misc;
mod oprt;
mod parser;
pub mod program;
mod stmt;
mod typed;

// Re-export parsers for convenient use
pub use self::decl::*;
pub use self::expr::*;
pub use self::misc::*;
pub use self::oprt::*;
pub use self::parser::*;
pub use self::stmt::*;
pub use self::typed::*;

// Re-export structs in "ir"
pub use super::ir::decl::*;
pub use super::ir::expr::*;
pub use super::ir::misc::*;
pub use super::ir::oprt::*;
pub use super::ir::program::*;
pub use super::ir::stmt::*;
pub use super::ir::typed::*;

// Re-export winnow parsers
pub use winnow::ascii::digit1;
pub use winnow::ascii::multispace0;
pub use winnow::ascii::space0;
pub use winnow::ascii::space1;
pub use winnow::combinator::alt;
pub use winnow::combinator::cut_err;
pub use winnow::combinator::empty;
pub use winnow::combinator::fail;
pub use winnow::combinator::opt;
pub use winnow::combinator::peek;
pub use winnow::combinator::preceded;
pub use winnow::combinator::repeat;
pub use winnow::combinator::separated;
pub use winnow::combinator::terminated;
pub use winnow::combinator::trace;
pub use winnow::dispatch;
pub use winnow::error::ContextError;
pub use winnow::error::ParserError;
pub use winnow::error::StrContext;
pub use winnow::stream::AsChar;
pub use winnow::stream::Compare;
pub use winnow::stream::SliceLen;
pub use winnow::stream::Stream;
pub use winnow::stream::StreamIsPartial;
pub use winnow::token::any;
pub use winnow::token::literal;
pub use winnow::token::one_of;
pub use winnow::token::take_until;
pub use winnow::token::take_while;
pub use winnow::PResult;
pub use winnow::Parser;
