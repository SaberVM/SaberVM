mod error_handling;
mod header;
mod parse;
mod pretty;
mod verify;

use std::fs;

use crate::header::CapabilityPool;
use crate::header::Error;
use crate::header::Stmt2::*;
use crate::header::TypeListPool;
use crate::header::TypePool;

fn go(
    code: &Vec<u8>,
    mut cap_pool: &mut CapabilityPool,
    mut type_pool: &mut TypePool,
    mut tl_pool: &mut TypeListPool,
) -> Result<(), Error> {
    let prog = parse::go(code)?;

    let stmts = verify::go(prog, &mut cap_pool, &mut type_pool, &mut tl_pool)?;
    let p = &stmts[0];
    let Func2(_, t, ops) = p;
    dbg!(pretty::typ(&t, &type_pool, &tl_pool, &cap_pool));
    for op in ops {
        println!("{}", pretty::op2(&op))
    }
    Ok(())
}

fn main() {
    let mut cap_pool = CapabilityPool(vec![]);
    let mut type_pool = TypePool(vec![]);
    let mut tl_pool = TypeListPool(vec![]);

    let code = fs::read("bin.svm").unwrap();

    let res = go(&code, &mut cap_pool, &mut type_pool, &mut tl_pool);

    error_handling::handle(res, &cap_pool, &type_pool, &tl_pool);
}
