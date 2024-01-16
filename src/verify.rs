use std::collections::HashMap;
use std::collections::VecDeque;

use crate::header::CTStackVal;
use crate::header::CTStackVal::*;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::CapabilityPool;
use crate::header::Id;
use crate::header::Kind::*;
use crate::header::OpCode1::*;
use crate::header::OpCode2;
use crate::header::OpCode2::*;
use crate::header::Region;
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

pub fn go(
    stmts: Vec<Stmt1>,
    cap_pool: &mut CapabilityPool,
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<Vec<Stmt2>, i32> {
    let mut out: Vec<Stmt2> = vec![];
    let stmts2: Vec<(Stmt2, Constraints)> = stmts
        .iter()
        .map(|stmt| pass(stmt, cap_pool, type_pool, tl_pool))
        .collect::<Result<Vec<_>,i32>>()
        .unwrap();
    let mut types: HashMap<i32, Type> = HashMap::new();
    for pair in &stmts2 {
        let (Func2(label, t, _), _) = pair;
        types.insert(*label, *t);
    }
    let mut constraints: Constraints = HashMap::new();
    for pair in stmts2 {
        // TODO: check that the expected types of global functions are their actual types
        let (stmt, c) = pair;
        constraints.extend(c);
        out.push(stmt);
    }
    return Ok(out);
}

type StackType = VecDeque<TypeRef>;
type CTStackType = Vec<CTStackVal>;
type Constraints = HashMap<i32,(StackType,CTStackType)>;

fn pass(
    stmt: &Stmt1,
    cap_pool: &mut CapabilityPool,
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<(Stmt2, HashMap<i32, (StackType, CTStackType)>), i32> {
    let mut ct_stack: CTStackType = vec![];
    let Func1(label, ops) = stmt;
    let mut iter = ops.iter();
    let mut arg_types: Vec<TypeRef> = vec![];
    let mut stack_type: StackType = VecDeque::from([]);
    let mut rvars: Vec<Id> = vec![];
    let mut capabilities: Vec<Capability> = vec![];
    let mut cvars: Vec<Capability> = vec![];
    let mut tvars: Vec<Id> = vec![];
    let mut exist_stack: Vec<Id> = vec![];
    let mut out: Vec<OpCode2> = vec![];
    let mut fresh_id = 0;
    let mut constraints = HashMap::new();
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
                        return Err(1);
                    }
                },
                Op1Region() => {
                    ct_stack.push(CTRegion(RegionVar(Id(*label, fresh_id))));
                    rvars.push(Id(*label, fresh_id));
                    fresh_id += 1;
                }
                Op1Heap() => ct_stack.push(CTRegion(Heap())),
                Op1Cap() => {
                    let var = CapVar(Id(*label, fresh_id));
                    let cap = cap_pool.add(vec![var]);
                    cvars.push(var);
                    ct_stack.push(CTCapability(cap));
                    fresh_id += 1;
                }
                Op1CapLE() => {
                    let mb_bound = ct_stack.pop();
                    match mb_bound {
                        Some(CTCapability(bound)) => {
                            let var = CapVarBounded(Id(*label, fresh_id), bound);
                            ct_stack.push(CTCapability(cap_pool.add(vec![var])));
                            cvars.push(var);
                            fresh_id += 1;
                        }
                        _ => return Err(2),
                    }
                }
                Op1Own() => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => {
                            ct_stack.push(CTCapability(cap_pool.add(vec![Owned(r)])))
                        }
                        _ => return Err(3),
                    }
                }
                Op1Read() => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => {
                            ct_stack.push(CTCapability(cap_pool.add(vec![NotOwned(r)])))
                        }
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
                        Some(CTRegion(r)) => ct_stack.push(CTType(type_pool.add(THandle(r)))),
                        _ => return Err(6),
                    }
                }
                Op1i32() => ct_stack.push(CTType(type_pool.add(Ti32()))),
                Op1End() => panic!("op-end found during verification"),
                Op1Mut() => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(type_pool.add(TMutable(t)))),
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
                            return Err(9);
                        }
                    }
                }
                Op1Arr() => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(type_pool.add(TArray(t)))),
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
                                Some(CTType(t)) => {
                                    ct_stack.push(CTType(type_pool.add(TExists(id, t))))
                                }
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
                            return Err(14);
                        }
                    }
                }
                Op1CTGet(n) => {
                    let l = ct_stack.len();
                    let i = usize::from(*n);
                    if l == 0 || l - 1 < i {
                        return Err(33);
                    }
                    let mb_x = ct_stack.get(l - i - 1);
                    match mb_x {
                        Some(x) => ct_stack.push(*x),
                        None => return Err(15),
                    }
                }
                Op1CTPop() => {
                    ct_stack.pop();
                }
                Op1Unpack() => {
                    let mb_ex = stack_type.pop_back();
                    match mb_ex {
                        Some(tr) => match type_pool.get(tr) {
                            TExists(_id, tr) => {
                                stack_type.push_back(*tr) // simply remove the quantifier, unbinding its variable
                            }
                            _ => return Err(31),
                        },
                        _ => return Err(32),
                    }
                }
                Op1Get(n) => {
                    let l = stack_type.len();
                    let i = usize::from(*n);
                    if l == 0 || l - 1 < i {
                        return Err(34);
                    }
                    match stack_type.get(l - 1 - i) {
                        Some(t) => stack_type.push_back(*t),
                        _ => return Err(17),
                    }
                    out.push(Op2Get(*n))
                }
                Op1Init(n) => {
                    let mb_val = stack_type.pop_back();
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(tpl) {
                            TTuple(ts, r) => match tl_pool.get(*ts).get(usize::from(*n)) {
                                None => return Err(20),
                                Some(formal) => match mb_val {
                                    None => return Err(21),
                                    Some(actual) => {
                                        if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                            println!("Type error! Incapable init");
                                            dbg!(r, &capabilities);
                                            return Err(35);
                                        }
                                        if type_pool.get(*formal) == type_pool.get(actual) {
                                            stack_type.push_back(tpl)
                                        } else {
                                            println!("Type error! init is setting a tuple field of the wrong type!");
                                            return Err(22);
                                        }
                                    }
                                },
                            },
                            _ => return Err(18),
                        },
                        None => return Err(19),
                    }
                    out.push(Op2Init(*n))
                }
                Op1Malloc() => {
                    let mb_t = ct_stack.pop();
                    let t = match mb_t {
                        Some(CTType(t)) => t,
                        _ => return Err(23),
                    };
                    let mb_rhandle = stack_type.pop_back();
                    match mb_rhandle {
                        Some(t) => match type_pool.get(t) {
                            THandle(r) => {
                                if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                    println!("Type error! Incapable malloc");
                                    dbg!(r, &capabilities);
                                    return Err(36);
                                }
                            }
                            _ => {
                                println!("Type error! malloc expects a region handle!");
                                return Err(25);
                            }
                        },
                        _ => return Err(24),
                    }
                    stack_type.push_back(t);
                    out.push(Op2Malloc(4)) // TODO: use actual size in bytes of t
                }
                Op1Proj(n) => {
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(tpl) {
                            TTuple(ts, r) => match tl_pool.get(*ts).get(usize::from(*n)) {
                                None => return Err(26),
                                Some(t) => {
                                    if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                        println!("Type error! Incapable region init");
                                        dbg!(r, &capabilities);
                                        return Err(37);
                                    }
                                    stack_type.push_back(*t);
                                }
                            },
                            _ => return Err(27),
                        },
                        None => return Err(28),
                    }
                    out.push(Op2Proj(*n))
                }
                Op1Clean(n) => {
                    let idx = i32::from(*n);
                    let diff = (stack_type.len() as i32) - 1 - idx;
                    if diff < 0 {
                        return Err(29);
                    }
                    let count = (if diff > 0xFF { 0xFF } else { diff }) as u8; // definitely won't lose information, since its in (0, 255)
                    for _ in 0..count {
                        stack_type.pop_front();
                    }
                    out.push(Op2Clean(*n, count))
                    // NOTE: this doesn't pop enough if the stack type is too large. Doing it successive times will work though.
                }
                Op1Call() => {
                    let mb_t = stack_type.pop_back();
                    match mb_t {
                        Some(tr) => {
                            let t = type_pool.get(tr);
                            match t {
                                TGuess(l) => {
                                    constraints.insert(*l, (stack_type.clone(), ct_stack.clone()));
                                }
                                TForall(id,k,tr) => todo!(),
                                TFunc(cr,tsr) => todo!(),
                                _ => return Err(39)
                            }
                        }
                        None => return Err(38)
                    }
                    out.push(Op2Call())
                }
            },
        }
    }
    if exist_stack.len() > 0 {
        return Err(16);
    }
    let t = tvars.iter().fold(
        TFunc(cap_pool.add(capabilities), tl_pool.add(arg_types)),
        |t, id| TForall(*id, KType, type_pool.add(t)),
    );
    let t = cvars.iter().fold(t, |t, c| match c {
        CapVar(id) => TForall(*id, KCapability(None), type_pool.add(t)),
        CapVarBounded(id, bound) => TForall(*id, KCapability(Some(*bound)), type_pool.add(t)),
        _ => panic!("nonvar in cvars"),
    });
    let t = rvars
        .iter()
        .fold(t, |t, r| TForall(*r, KRegion, type_pool.add(t)));
    Ok((Func2(*label, t, out), constraints))
}

pub fn capable(r: &Region, c: &Capability, cap_pool: &CapabilityPool) -> bool {
    match c {
        Owned(r2) | NotOwned(r2) if *r == *r2 => true,
        CapVarBounded(_, cr) => cap_pool.get(*cr).iter().any(|c| capable(r, c, cap_pool)),
        _ => false,
    }
}
