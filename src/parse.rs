/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::*;

/// Output of the lexer, input of the parser.
/// A sequence of (possibly parameterized) opcodes.
type LexedOpcodes = Vec<Op1>;

/// Lex bytes into (possibly parameterized) intructions.
fn lex(bytes: &ByteStream) -> Result<(usize, Vec<u8>, LexedOpcodes, u32), Error> {
    let mut bytes_iter = bytes.iter();
    let mut lexed_opcodes = vec![];
    let mut data_section_len_vec: [u8; 4] = [0, 0, 0, 0];
    for i in 0..4 {
        let Some(a) = bytes_iter.next() else {
            dbg!("a");
            return Err(Error::UnexpectedEOF);
        };
        data_section_len_vec[i] = *a;
    }
    let data_section_len_u32 = u32::from_le_bytes(data_section_len_vec);
    let data_section_len = data_section_len_u32 as usize;
    // skip past whatever bytes are in the data section
    let mut data_section = Vec::with_capacity(data_section_len);
    for _ in 0..data_section_len {
        data_section.push(
            *(bytes_iter.next().ok_or_else(|| {
                dbg!(data_section_len);
                Error::UnexpectedEOF
            })?),
        );
    }
    let mut a = [0, 0, 0, 0];
    for i in 0..4 {
        match bytes_iter.next() {
            None => {
                dbg!("c");
                return Err(Error::UnexpectedEOF);
            }
            Some(b) => {
                a[i] = *b;
            }
        }
    }
    let mut pos = 8 + data_section_len_u32;
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
                0x18 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::NewRgn(
                                    (*n4 as u32) << 24
                                        | (*n3 as u32) << 16
                                        | (*n2 as u32) << 8
                                        | (*n1 as u32),
                                ),
                            },
                        },
                    },
                },
                0x19 => Op1::FreeRgn,
                0x1A => Op1::Ptr,
                0x1B => Op1::Deref,
                0x1C => Op1::Arr,
                0x1D => Op1::ArrMut,
                0x1E => Op1::ArrProj,
                0x1F => Op1::Add,
                0x20 => Op1::Mul,
                0x21 => Op1::Div,
                0x22 => Op1::CallNZ,
                0x23 => match bytes_iter.next() {
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n1) => match bytes_iter.next() {
                        None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                        Some(n2) => match bytes_iter.next() {
                            None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                            Some(n3) => match bytes_iter.next() {
                                None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                                Some(n4) => Op1::Data(
                                    (*n4 as u32) << 24
                                        | (*n3 as u32) << 16
                                        | (*n2 as u32) << 8
                                        | (*n1 as u32),
                                ),
                            },
                        },
                    },
                },
                0x24 => Op1::DataSec,
                0x25 => Op1::U8,
                0x26 => Op1::PrintN,
                0x27 => match bytes_iter.next() {
                    Some(n) => Op1::U8Lit(*n),
                    None => return Err(Error::SyntaxErrorParamNeeded(pos, *byte)),
                }
                0x28 => Op1::U8ToI32,
                op => return Err(Error::SyntaxErrorUnknownOp(pos, *op)),
            }),
        }
        pos += 1;
    }
    Ok((data_section_len, data_section, lexed_opcodes, n))
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
                None => {
                    dbg!("a");
                    return Err(Error::UnexpectedEOF)
                }
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
                    break;
                }
                Some(Op1::CallNZ) => {
                    current_stmt_opcodes.push(Op1::CallNZ);
                    break;
                }
                Some(Op1::Halt) => {
                    current_stmt_opcodes.push(Op1::Halt);
                    break;
                }
                Some(op) => current_stmt_opcodes.push(*op),
            }
        }
        parsed_stmts.push(Stmt1::Func(i, current_stmt_opcodes));
        current_stmt_opcodes = vec![];
    }
    if current_stmt_opcodes.len() > 0 {
        dbg!(current_stmt_opcodes);
        return Err(Error::UnexpectedEOF);
    }
    Ok(parsed_stmts)
}

/// Lex a stream of bytes, maybe return an error, otherwise parse.
pub fn go(istream: &ByteStream) -> Result<(usize, Vec<u8>, Vec<ForwardDec>, Vec<Stmt1>), Error> {
    // this is two-pass currently (lex and parse); it would be straightforward to fuse these passes.
    let (data_decs, data_section, tokens, n) = lex(istream)?;
    let (forward_decs, rest) = parse_forward_decs(&tokens, n)?;
    let stmts = parse(rest, n)?;
    Ok((data_decs, data_section, forward_decs, stmts))
}
