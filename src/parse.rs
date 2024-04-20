/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::*;

/// Output of the lexer, input of the parser.
/// A sequence of (possibly parameterized) opcodes.
type LexedOpcodes = Vec<Op1>;

const BYTES_TO_SKIP: u32 = 4;

/// Lex bytes into (possibly parameterized) intructions.
fn lex(bytes: &ByteStream) -> Result<LexedOpcodes, Error> {
    let mut bytes_iter = bytes.iter();
    let mut lexed_opcodes = vec![];
    for _ in 0..BYTES_TO_SKIP {
        bytes_iter.next();
    }
    let mut pos = BYTES_TO_SKIP;
    loop {
        match bytes_iter.next() {
            None => break,
            Some(byte) => lexed_opcodes.push(match byte {
                0x00 => Op1::Req,
                0x01 => Op1::Region,
                0x02 => Op1::Unique,
                0x03 => Op1::Handle,
                0x04 => Op1::I32,
                0x05 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Tuple(*n),
                },
                0x06 => Op1::Quantify,
                0x07 => Op1::Some,
                0x08 => Op1::All,
                0x09 => Op1::Rgn,
                0x0A => Op1::End,
                0x0B => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Func(*n),
                },
                0x0C => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::CTGet(*n),
                },
                0x0D => Op1::Unpack,
                0x0E => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Get(*n),
                },
                0x0F => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Init(*n),
                },
                0x10 => Op1::Malloc,
                0x11 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Proj(*n),
                },
                0x12 => Op1::Call,
                0x13 => Op1::Print,
                0x14 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::Lit(
                                    ((*n1 as u32) << 24
                                        | (*n2 as u32) << 16
                                        | (*n3 as u32) << 8
                                        | (*n4 as u32)) as i32,
                                ),
                            },
                        },
                    },
                },
                0x15 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::GlobalFunc(
                                    (*n1 as u32) << 24
                                        | (*n2 as u32) << 16
                                        | (*n3 as u32) << 8
                                        | (*n4 as u32),
                                ),
                            },
                        },
                    },
                },
                0x16 => Op1::Halt,
                0x17 => Op1::Pack,
                0x18 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::Size(
                                    (*n1 as u32) << 24
                                        | (*n2 as u32) << 16
                                        | (*n3 as u32) << 8
                                        | (*n4 as u32),
                                ),
                            },
                        },
                    },
                },
                0x19 => Op1::NewRgn,
                0x1A => Op1::FreeRgn,
                0x1B => Op1::Ptr,
                0x1C => Op1::Deref,
                op => return Err(Error::SyntaxErrorUnknownOp(pos, *op)),
            }),
        }
        pos += 1;
    }
    Ok(lexed_opcodes)
}

/// Divide an opcode stream into functions, producing the AST.
fn parse(tokens: &LexedOpcodes) -> ParsedStmts {
    let mut parsed_stmts = vec![];
    let mut current_stmt_opcodes = vec![];
    let mut tokens_iter = tokens.iter();
    let mut line = 0;
    let mut function_label = 0;
    loop {
        match tokens_iter.next() {
            None => break,
            Some(Op1::Call) => {
                current_stmt_opcodes.push(Op1::Call);
                parsed_stmts.push(Stmt1::Func(function_label, current_stmt_opcodes));
                line += 1;
                function_label = line;
                current_stmt_opcodes = vec![];
            }
            Some(Op1::Halt) => {
                current_stmt_opcodes.push(Op1::Halt);
                parsed_stmts.push(Stmt1::Func(function_label, current_stmt_opcodes));
                line += 1;
                function_label = line;
                current_stmt_opcodes = vec![];
            }
            Some(op) => current_stmt_opcodes.push(*op),
        }
        line += 1;
    }
    if current_stmt_opcodes.len() > 0 {
        parsed_stmts.push(Stmt1::Func(function_label, current_stmt_opcodes));
    }
    parsed_stmts
}

/// Lex a stream of bytes, maybe return an error, otherwise parse.
pub fn go(istream: &ByteStream) -> Result<ParsedStmts, Error> {
    let tokens = lex(istream)?;
    Ok(parse(&tokens)) // this is two-pass currently (lex and parse); it would be straightforward to fuse these passes.
}
