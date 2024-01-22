mod error_handling;
mod header;
mod parse;
mod pretty;
mod verify;

use std::fs;

use crate::header::Error;
use crate::header::Stmt2::*;
use crate::header::TypeListPool;
use crate::header::TypePool;

fn go(
    code: &Vec<u8>,
    mut type_pool: &mut TypePool,
    mut tl_pool: &mut TypeListPool,
) -> Result<(), Error> {
    let prog = parse::go(code)?;

    let stmts = verify::go(prog, &mut type_pool, &mut tl_pool)?;
    let p = &stmts[0];
    let Func2(_, tr, ops) = p;
    dbg!(pretty::typ(&tr, &type_pool, &tl_pool));
    for op in ops {
        println!("{}", pretty::op2(&op))
    }
    Ok(())
}

fn main() {
    let mut type_pool = TypePool(vec![]);
    let mut tl_pool = TypeListPool(vec![]);

    let code = fs::read("bin.svm").unwrap();

    let res = go(&code, &mut type_pool, &mut tl_pool);

    error_handling::handle(res, &type_pool, &tl_pool);
}
