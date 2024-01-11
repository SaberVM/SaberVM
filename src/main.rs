
mod header;
mod parse;
mod verify;

use crate::header::OpCode1::*;
use crate::header::Stmt1::*;
use crate::header::CapabilityPool;
use crate::header::TypePool;
use crate::header::TypeListPool;

fn main() {
  // let code: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x01];
  // dbg!(&code);
  // let prog = parse::go(&code);
  // dbg!(&prog);
  let stmt = &Func1(0, vec![
    Op1Region(),
    Op1Cap(),
    Op1CTGet(0),
    Op1Req(),
    Op1All(),
    Op1CTGet(0),
    Op1Req(),
    Op1CTGet(2),
    Op1Some(),
    Op1CTGet(3),
    Op1CTGet(5),
    Op1CTGet(4),
    Op1CTGet(0),
    Op1Tuple(2),
    Op1CTGet(2),
    Op1Func(2),
    Op1Tuple(2),
    Op1Emos(),
    Op1Req(),
    Op1CTGet(2),
    Op1Handle(),
    Op1Req(),
    Op1CTGet(2),
    Op1CTGet(1),
    Op1CTGet(0),
    Op1Tuple(2),
    Op1Get(2),
    Op1Malloc(),
    Op1Get(1),
    Op1Init(0),
    Op1Get(1),
    Op1Init(1),
    Op1Get(2),
    Op1Proj(1),
    Op1Get(3),
    Op1Proj(0),
    Op1Clean(3),
    Op1Call()
  ]);
  let cap_pool = CapabilityPool(vec![]);
  let type_pool = TypePool(vec![]);
  let tl_pool = TypeListPool(vec![]);

  let prog2 = verify::go(stmt, cap_pool, type_pool, tl_pool);
  dbg!(prog2);
}
