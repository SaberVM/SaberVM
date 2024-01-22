use crate::header::Error;
use crate::header::Error::*;
use crate::header::OpCode1;
use crate::header::OpCode1::*;
use crate::header::Stmt1;
use crate::header::Stmt1::*;

fn lex(istream: &Vec<u8>) -> Result<Vec<OpCode1>, Error> {
    let mut i = istream.iter();
    let mut out = vec![];
    i.next();
    i.next();
    i.next();
    i.next();
    loop {
        match i.next() {
            None => break,
            Some(byte) => out.push(match byte {
                0x00 => Op1Req,
                0x01 => Op1Region,
                0x02 => Op1Heap,
                0x03 => Op1Cap,
                0x04 => Op1CapLE,
                0x05 => Op1Own,
                0x06 => Op1Read,
                0x07 => Op1Both,
                0x08 => Op1Handle,
                0x09 => Op1i32,
                0x0A => Op1End,
                0x0B => Op1Mut,
                0x0C => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1Tuple(*n),
                },
                0x0D => Op1Arr,
                0x0E => Op1All,
                0x0F => Op1Some,
                0x10 => Op1Emos,
                0x11 => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1Func(*n),
                },
                0x12 => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1CTGet(*n),
                },
                0x13 => Op1CTPop,
                0x14 => Op1Unpack,
                0x15 => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1Get(*n),
                },
                0x16 => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1Init(*n),
                },
                0x17 => Op1Malloc,
                0x18 => match i.next() {
                    None => return Err(SyntaxErrorParamNeeded(*byte)),
                    Some(n) => Op1Proj(*n),
                },
                0x19 => Op1Call,
                op => return Err(SyntaxErrorUnknownOp(*op)),
            }),
        }
    }
    Ok(out)
}

fn parse(tokens: &Vec<OpCode1>) -> Vec<Stmt1> {
    let mut out = vec![];
    let mut curr = vec![];
    let mut iter = tokens.iter();
    let mut i = 4;
    loop {
        match iter.next() {
            None => break,
            Some(Op1End) => {
                out.push(Func1(i, curr));
                curr = vec![];
            }
            Some(op) => curr.push(*op),
        }
        i += 1;
    }
    out.push(Func1(i, curr));
    out
}

pub fn go(istream: &Vec<u8>) -> Result<Vec<Stmt1>, Error> {
    let tokens = lex(istream)?;
    Ok(parse(&tokens))
}
