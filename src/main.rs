mod error_handling;
mod header;
mod parse;
mod pretty;
mod verify;

use std::fs;

use crate::header::Error;
use crate::header::Stmt2::*;

fn go(
    code: &Vec<u8>,
) -> Result<(), Error> {
    let prog = parse::go(code)?;

    let stmts = verify::go(prog)?;
    let p = &stmts[0];
    let Func2(_, t, ops) = p;
    dbg!(pretty::typ(&t));
    for op in ops {
        println!("{}", pretty::op2(&op))
    }
    Ok(())
}

fn main() {
    let code = fs::read("bin.svm").unwrap();

    let res = go(&code);

    error_handling::handle(res);
}
