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
fn lex(bytes: &ByteStream) -> Result<(LexedOpcodes, u32), Error> {
    let mut bytes_iter = bytes.iter();
    let mut lexed_opcodes = vec![];
    for _ in 0..BYTES_TO_SKIP {
        bytes_iter.next();
    }
    let mut pos = BYTES_TO_SKIP;
    let mut a = [0, 0, 0, 0];
    for i in 0..4 {
        match bytes_iter.next() {
            None => panic!("unexpected eof (this shouldn't be a panic tbh)"),
            Some(b) => {
                a[i] = *b;
                pos += 1;
            }
        }
    }
    let n = u32::from_le_bytes(a);
    loop {
        match bytes_iter.next() {
            None => break,
            Some(byte) => lexed_opcodes.push(match byte {
                0x00 => Op1::Unique,
                0x01 => Op1::Handle,
                0x02 => Op1::I32,
                0x03 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Tuple(*n),
                },
                0x04 => Op1::Some,
                0x05 => Op1::All,
                0x06 => Op1::Rgn,
                0x07 => Op1::End,
                0x08 => Op1::App,
                0x09 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Func(*n),
                },
                0x0A => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::CTGet(*n),
                },
                0x0B => Op1::Lced,
                0x0C => Op1::Unpack,
                0x0D => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Get(*n),
                },
                0x0E => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Init(*n),
                },
                0x0F => Op1::Malloc,
                0x10 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1::Proj(*n),
                },
                0x11 => Op1::Call,
                0x12 => Op1::Print,
                0x13 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::Lit(
                                    ((*n4 as u32) << 24
                                        | (*n3 as u32) << 16
                                        | (*n2 as u32) << 8
                                        | (*n1 as u32)) as i32,
                                ),
                            },
                        },
                    },
                },
                0x14 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::GlobalFunc(
                                    (*n4 as u32) << 24
                                        | (*n3 as u32) << 16
                                        | (*n2 as u32) << 8
                                        | (*n1 as u32),
                                ),
                            },
                        },
                    },
                },
                0x15 => Op1::Halt,
                0x16 => Op1::Pack,
                0x17 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::Size(
                                    (*n4 as u32) << 24
                                        | (*n3 as u32) << 16
                                        | (*n2 as u32) << 8
                                        | (*n1 as u32),
                                ),
                            },
                        },
                    },
                },
                0x18 => Op1::NewRgn,
                0x19 => Op1::FreeRgn,
                0x1A => Op1::Ptr,
                0x1B => Op1::Deref,
                op => return Err(Error::SyntaxErrorUnknownOp(pos, *op)),
            }),
        }
        pos += 1;
    }
    Ok((lexed_opcodes, n))
}

fn parse_forward_decs(
    tokens: &LexedOpcodes,
    n: u32,
) -> Result<(Vec<ForwardDec>, std::slice::Iter<'_, Op1>), Error> {
    let mut forward_decs = vec![];
    let mut tokens_iter = tokens.iter();
    let mut current_stmt_opcodes = vec![];
    for i in 0..n {
        loop {
            match tokens_iter.next() {
                None => panic!("this shouldn't be a panic"),
                Some(Op1::Lced) => break,
                Some(op) => current_stmt_opcodes.push(*op),
            }
        }
        forward_decs.push(ForwardDec::Func(i, current_stmt_opcodes));
        current_stmt_opcodes = vec![];
    }
    Ok((forward_decs, tokens_iter))
}

/// Divide an opcode stream into functions, producing the AST.
fn parse(mut tokens_iter: std::slice::Iter<'_, Op1>, n: u32) -> Result<Vec<Stmt1>, Error> {
    let mut parsed_stmts = vec![];
    let mut current_stmt_opcodes = vec![];
    for i in 0..n {
        loop {
            match tokens_iter.next() {
                None => break,
                Some(Op1::Call) => {
                    current_stmt_opcodes.push(Op1::Call);
                    parsed_stmts.push(Stmt1::Func(i, current_stmt_opcodes));
                    current_stmt_opcodes = vec![];
                }
                Some(Op1::Halt) => {
                    current_stmt_opcodes.push(Op1::Halt);
                    parsed_stmts.push(Stmt1::Func(i, current_stmt_opcodes));
                    current_stmt_opcodes = vec![];
                }
                Some(op) => current_stmt_opcodes.push(*op),
            }
        }
    }
    if current_stmt_opcodes.len() > 0 {
        dbg!(current_stmt_opcodes);
        panic!("this shouldn't be a panic either");
    }
    Ok(parsed_stmts)
}

/// Lex a stream of bytes, maybe return an error, otherwise parse.
pub fn go(istream: &ByteStream) -> Result<(Vec<ForwardDec>, Vec<Stmt1>), Error> {
    // this is two-pass currently (lex and parse); it would be straightforward to fuse these passes.
    let (tokens, n) = lex(istream)?;
    let (forward_decs, rest) = parse_forward_decs(&tokens, n)?;
    let stmts = parse(rest, n)?;
    Ok((forward_decs, stmts))
}
