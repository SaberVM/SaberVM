use std::collections::VecDeque;

use crate::header::CTStackVal;
use crate::header::CTStackVal::*;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::CapabilityPool;
use crate::header::Id;
use crate::header::OpCode1::*;
use crate::header::OpCode2::*;
use crate::header::Region::*;
use crate::header::Stmt1;
use crate::header::Stmt1::*;
use crate::header::Stmt2;
use crate::header::Stmt2::*;
use crate::header::Type;
use crate::header::Type::*;
use crate::header::TypeListPool;
use crate::header::TypePool;
use crate::header::TypeRef;

pub fn go<'a>(stmt: &Stmt1, mut cap_pool: CapabilityPool, mut type_pool: TypePool, mut tl_pool: TypeListPool) -> Result<Stmt2, i32> {
  let mut ct_stack: Vec<CTStackVal> = vec![];
  let Func1(label, ops) = stmt;
  let mut iter = ops.iter();
  let mut arg_types: Vec<TypeRef> = vec![];
  let mut stack_type: VecDeque<TypeRef> = VecDeque::from([]);
  let mut regions: Vec<Id> = vec![];
  let mut capabilities: Vec<Capability> = vec![];
  let mut tvars: Vec<Id> = vec![];
  let mut exist_stack: Vec<Id> = vec![];
  let mut out = vec![];
  let mut fresh_id = 0;
  loop {
    match iter.next() {
      None => break,
      Some(op) => match op {
        Op1Req() => match ct_stack.pop() {
          None => return Err(0),
          Some(CTType(t)) => {
            arg_types.push(t);
            stack_type.push_front(t);
          }
          Some(CTCapability(c)) => capabilities.extend(cap_pool.get(c)),
          Some(x) => {
            dbg!(x);
            return Err(1)
          }
        },
        Op1Region() => {
          ct_stack.push(CTRegion(RegionVar(Id(
            *label, fresh_id,
          ))));
          regions.push(Id(*label, fresh_id));
          fresh_id += 1;
        }
        Op1Heap() => ct_stack.push(CTRegion(Heap())),
        Op1Cap() => {
          let cap =
            cap_pool.add(vec![CapVar(Id(*label, fresh_id))]);
          ct_stack.push(CTCapability(cap));
          fresh_id += 1;
        }
        Op1CapLE() => {
          let mb_bound = ct_stack.pop();
          match mb_bound {
            Some(CTCapability(bound)) => {
              ct_stack.push(CTCapability(cap_pool.add(vec![
                CapVarBounded(Id(*label, fresh_id), bound),
              ])));
              fresh_id += 1;
            }
            _ => return Err(2),
          }
        }
        Op1Own() => {
          let mb_r = ct_stack.pop();
          match mb_r {
            Some(CTRegion(r)) => ct_stack.push(CTCapability(
              cap_pool.add(vec![Owned(r)]),
            )),
            _ => return Err(3),
          }
        }
        Op1Read() => {
          let mb_r = ct_stack.pop();
          match mb_r {
            Some(CTRegion(r)) => ct_stack.push(CTCapability(
              cap_pool.add(vec![ReadOnly(r)]),
            )),
            _ => return Err(3),
          }
        }
        Op1Both() => {
          let mb_c1 = ct_stack.pop();
          match mb_c1 {
            Some(CTCapability(c1)) => {
              let mb_c2 = ct_stack.pop();
              match mb_c2 {
                Some(CTCapability(c2)) => {
                  let c1 = cap_pool.get(c1);
                  let c2 = cap_pool.get(c2);
                  ct_stack.push(CTCapability(
                    cap_pool.add([&c1[..], &c2[..]].concat()),
                  ))
                }
                _ => return Err(5),
              }
            }
            _ => return Err(4),
          }
        }
        Op1Handle() => {
          let mb_r = ct_stack.pop();
          match mb_r {
            Some(CTRegion(r)) => {
              ct_stack.push(CTType(type_pool.add(THandle(r))))
            }
            _ => return Err(6),
          }
        }
        Op1i32() => ct_stack.push(CTType(type_pool.add(Ti32()))),
        Op1End() => panic!("op-end found during verification"),
        Op1Mut() => {
          let mb_t = ct_stack.pop();
          match mb_t {
            Some(CTType(t)) => {
              ct_stack.push(CTType(type_pool.add(TMutable(t))))
            }
            _ => return Err(7),
          }
        }
        Op1Tuple(n) => {
          let mut ts = vec![];
          for _ in 0..*n {
            let mb_t = ct_stack.pop();
            match mb_t {
              Some(CTType(t)) => ts.push(t),
              _ => return Err(8),
            }
          }
          let mb_r = ct_stack.pop();
          match mb_r {
            Some(CTRegion(r)) => {
              ct_stack.push(CTType(type_pool.add(TTuple(tl_pool.add(ts), r))))
            }
            x => {
              dbg!(x);
              return Err(9)
            }
          }
        }
        Op1Arr() => {
          let mb_t = ct_stack.pop();
          match mb_t {
            Some(CTType(t)) => {
              ct_stack.push(CTType(type_pool.add(TArray(t))))
            }
            _ => return Err(10),
          }
        }
        Op1All() => {
          ct_stack.push(CTType(type_pool.add(TVar(Id(*label, fresh_id)))));
          tvars.push(Id(*label, fresh_id));
          fresh_id += 1
        }
        Op1Some() => {
          ct_stack.push(CTType(type_pool.add(TVar(Id(*label, fresh_id)))));
          exist_stack.push(Id(*label, fresh_id));
          fresh_id += 1;
        }
        Op1Emos() => {
          let mb_var = exist_stack.pop();
          match mb_var {
            None => return Err(11),
            Some(id) => {
              let mb_t = ct_stack.pop();
              match mb_t {
                Some(CTType(t)) => ct_stack
                  .push(CTType(type_pool.add(TExists(id, t)))),
                _ => return Err(12),
              }
            }
          }
        }
        Op1Func(n) => {
          let mut ts = vec![];
          for _ in 0..*n {
            let mb_t = ct_stack.pop();
            match mb_t {
              Some(CTType(t)) => ts.push(t),
              _ => return Err(13),
            }
          }
          let mb_c = ct_stack.pop();
          match mb_c {
            Some(CTCapability(c)) => {
              ct_stack.push(CTType(type_pool.add(TFunc(c, tl_pool.add(ts)))))
            }
            x => {
              dbg!(x);
              return Err(14)
            }
          }
        }
        Op1CTGet(n) => {
          let mb_x = ct_stack.get(ct_stack.len() - usize::from(*n) - 1);
          match mb_x {
            Some(x) => ct_stack.push(*x),
            None => return Err(15),
          }
        }
        Op1CTPop() => {
          ct_stack.pop();
        }
        Op1Get(n) => out.push(Op2Get(*n)),
        Op1Init(n) => out.push(Op2Init(*n)),
        Op1Malloc() => {
          let _t = ct_stack.pop();
          out.push(Op2Malloc(4)) // TODO: use actual size in bytes of t
        }
        Op1Proj(n) => out.push(Op2Proj(*n)),
        Op1Clean(n) => out.push(Op2Clean(*n, 0)),
        Op1Call() => out.push(Op2Call()),
      },
    }
  }
  if exist_stack.len() > 0 {
    return Err(16);
  }
  dbg!(capabilities);
  dbg!(regions);
  dbg!(tvars);
  let t: Vec<&Type> = stack_type.iter().rev().map(|x|type_pool.get(*x)).collect();
  dbg!(t);
  Ok(Func2(*label, out))
}
