use crate::header::Error;
use crate::header::Error::*;
use crate::header::OpCode1;
use crate::header::OpCode1::*;
use crate::header::Stmt1;
use crate::header::Stmt1::*;

type ByteStream = Vec<u8>; // not streamed currently
type LexedOpcodes = Vec<OpCode1>;
type ParsedStmts = Vec<Stmt1>;

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
                0x0C => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1Tuple(*n),
                },
                0x0D => Op1Arr,
                0x0E => Op1All,
                0x0F => Op1Some,
                0x10 => Op1Emos,
                0x11 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1Func(*n),
                },
                0x12 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1CTGet(*n),
                },
                0x13 => Op1CTPop,
                0x14 => Op1Unpack,
                0x15 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1Get(*n),
                },
                0x16 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1Init(*n),
                },
                0x17 => Op1Malloc,
                0x18 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => Op1Proj(*n),
                },
                0x19 => Op1Call,
                op => return Err(SyntaxErrorUnknownOp(pos, *op)),
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
    let mut byte_pos = BYTES_TO_SKIP;
    let mut function_label = byte_pos;
    loop {
        match tokens_iter.next() {
            None => break,
            Some(Op1End) => {
                parsed_stmts.push(Func1(function_label, current_stmt_opcodes));
                function_label = byte_pos;
                current_stmt_opcodes = vec![];
            }
            Some(op) => current_stmt_opcodes.push(*op),
        }
        byte_pos += 1;
    }
    parsed_stmts.push(Func1(function_label, current_stmt_opcodes));
    parsed_stmts
}

/// Lex a stream of bytes, maybe return an error, otherwise parse.
pub fn go(istream: &ByteStream) -> Result<ParsedStmts, Error> {
    let tokens = lex(istream)?;
    Ok(parse(&tokens)) // this is two-pass currently (lex and parse); it would be straightforward to fuse these passes.
}

#[cfg(test)]
mod tests {
    use crate::header::Stmt1::*;
    use crate::parse;
    use crate::header::OpCode1::*;
    use crate::header::Error::*;
    
    #[test]
    fn test_lex() {
        let input = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x03];
        let output = parse::lex(&input);
        assert_eq!(Ok(vec![Op1Req, Op1CTGet(3)]), output);
    }

    #[test]
    fn test_lex_bad() {
        let input = vec![0x00, 0x00, 0x00, 0x00, 0x12];
        let output = parse::lex(&input);
        assert_eq!(Err(SyntaxErrorParamNeeded(0, 0x12)), output);
    }

    #[test]
    fn test_parse() {
        let input = vec![Op1Req, Op1End, Op1Region];
        
        let output = parse::parse(&input);

        let Some(stmt1) = output.get(0) else {panic!()};
        let Func1(4, ops1) = stmt1 else {panic!()};
        assert!(ops1.len() == 1);

        let Some(stmt2) = output.get(1) else {panic!()};
        let Func1(5, ops2) = stmt2 else {panic!()};
        assert!(ops2.len() == 1);
    }
}