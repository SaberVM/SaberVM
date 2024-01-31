use crate::header::ByteStream;
use crate::header::Error;
use crate::header::Error::*;
use crate::header::ParsedStmts;
use crate::header::UnverifiedOpcode;
use crate::header::UnverifiedOpcode::*;
use crate::header::UnverifiedStmt;

/// Output of the lexer, input of the parser.
/// A sequence of (possibly parameterized) opcodes.
type LexedOpcodes = Vec<UnverifiedOpcode>;

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
                0x00 => ReqOp,
                0x01 => RegionOp,
                0x02 => HeapOp,
                0x03 => CapOp,
                0x04 => CapLEOp,
                0x05 => UniqueOp,
                0x06 => RWOp,
                0x07 => BothOp,
                0x08 => HandleOp,
                0x09 => I32Op,
                0x0A => EndFunctionOp,
                0x0B => MutOp,
                0x0C => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => TupleOp(*n),
                },
                0x0D => ArrOp,
                0x0E => AllOp,
                0x0F => SomeOp,
                0x10 => EmosOp,
                0x11 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => FuncOp(*n),
                },
                0x12 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => CTGetOp(*n),
                },
                0x13 => CTPopOp,
                0x14 => UnpackOp,
                0x15 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => GetOp(*n),
                },
                0x16 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => InitOp(*n),
                },
                0x17 => MallocOp,
                0x18 => match bytes_iter.next() {
                    None => return Err(SyntaxErrorParamNeeded(pos, *byte)),
                    Some(n) => ProjOp(*n),
                },
                0x19 => CallOp,
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
            Some(EndFunctionOp) => {
                parsed_stmts.push(UnverifiedStmt::Func(function_label, current_stmt_opcodes));
                function_label = byte_pos;
                current_stmt_opcodes = vec![];
            }
            Some(op) => current_stmt_opcodes.push(*op),
        }
        byte_pos += 1;
    }
    parsed_stmts.push(UnverifiedStmt::Func(function_label, current_stmt_opcodes));
    parsed_stmts
}

/// Lex a stream of bytes, maybe return an error, otherwise parse.
pub fn go(istream: &ByteStream) -> Result<ParsedStmts, Error> {
    let tokens = lex(istream)?;
    Ok(parse(&tokens)) // this is two-pass currently (lex and parse); it would be straightforward to fuse these passes.
}

#[cfg(test)]
mod tests {
    use crate::header::Error::*;
    use crate::header::UnverifiedOpcode::*;
    use crate::header::UnverifiedStmt;
    use crate::parse;

    #[test]
    fn test_lex() {
        let input = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x03];
        let output = parse::lex(&input);
        assert_eq!(Ok(vec![ReqOp, CTGetOp(3)]), output);
    }

    #[test]
    fn test_lex_bad() {
        let input = vec![0x00, 0x00, 0x00, 0x00, 0x12];
        let output = parse::lex(&input);
        assert_eq!(Err(SyntaxErrorParamNeeded(0, 0x12)), output);
    }

    #[test]
    fn test_parse() {
        let input = vec![ReqOp, EndFunctionOp, RegionOp];

        let output = parse::parse(&input);

        let Some(stmt1) = output.get(0) else { panic!() };
        let UnverifiedStmt::Func(4, ops1) = stmt1 else {
            panic!()
        };
        assert!(ops1.len() == 1);

        let Some(stmt2) = output.get(1) else { panic!() };
        let UnverifiedStmt::Func(5, ops2) = stmt2 else {
            panic!()
        };
        assert!(ops2.len() == 1);
    }
}
