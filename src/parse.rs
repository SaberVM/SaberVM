use crate::header::OpCode1;
use crate::header::Stmt1;

fn lex(istream: &Vec<u8>) -> Result<Vec<OpCode1>, i32> {
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
                0x00 => OpCode1::Op1Req(),
                0x01 => OpCode1::Op1Region(),
                0x02 => OpCode1::Op1Heap(),
                0x03 => OpCode1::Op1Cap(),
                0x04 => OpCode1::Op1CapLE(),
                0x05 => OpCode1::Op1Own(),
                0x06 => OpCode1::Op1Read(),
                0x07 => OpCode1::Op1Both(),
                0x08 => OpCode1::Op1Handle(),
                0x09 => OpCode1::Op1i32(),
                0x0A => OpCode1::Op1End(),
                0x0B => OpCode1::Op1Mut(),
                0x0C => match i.next() {
                    None => return Err(1),
                    Some(n) => OpCode1::Op1Tuple(*n),
                },
                0x0D => OpCode1::Op1Arr(),
                0x0E => OpCode1::Op1All(),
                0x0F => OpCode1::Op1Some(),
                0x10 => OpCode1::Op1Emos(),
                0x11 => match i.next() {
                    None => return Err(2),
                    Some(n) => OpCode1::Op1Func(*n),
                },
                0x12 => match i.next() {
                    None => return Err(3),
                    Some(n) => OpCode1::Op1CTGet(*n),
                },
                0x13 => OpCode1::Op1CTPop(),
                0x14 => match i.next() {
                    None => return Err(5),
                    Some(n) => OpCode1::Op1Get(*n),
                },
                0x15 => match i.next() {
                    None => return Err(6),
                    Some(n) => OpCode1::Op1Init(*n),
                },
                0x16 => OpCode1::Op1Malloc(),
                0x17 => match i.next() {
                    None => return Err(7),
                    Some(n) => OpCode1::Op1Proj(*n),
                },
                0x18 => match i.next() {
                    None => return Err(8),
                    Some(n) => OpCode1::Op1Clean(*n),
                },
                0x19 => OpCode1::Op1Call(),
                _ => return Err(0),
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
            Some(OpCode1::Op1End()) => {
                out.push(Stmt1::Func1(i, curr));
                curr = vec![];
            }
            Some(op) => curr.push(*op),
        }
        i += 1;
    }
    out.push(Stmt1::Func1(i, curr));
    out
}

pub fn go(istream: &Vec<u8>) -> Result<Vec<Stmt1>, i32> {
    let tokens = lex(istream)?;
    Ok(parse(&tokens))
}
