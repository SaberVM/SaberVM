use std::collections::HashMap;
use std::collections::VecDeque;

use crate::header::CTStackVal;
use crate::header::CTStackVal::*;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::CapabilityPool;
use crate::header::CapabilityRef;
use crate::header::Error;
use crate::header::Error::*;
use crate::header::Id;
use crate::header::Kind;
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
use crate::header::TypeListRef;
use crate::header::TypePool;
use crate::header::TypeRef;
use crate::header::get_kind;

pub fn go(
    stmts: Vec<Stmt1>,
    cap_pool: &mut CapabilityPool,
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<Vec<Stmt2>, Error> {
    let mut out: Vec<Stmt2> = vec![];
    let stmts2: Vec<(Stmt2, Constraints)> = stmts
        .iter()
        .map(|stmt| pass(stmt, cap_pool, type_pool, tl_pool))
        .collect::<Result<Vec<_>, Error>>()
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
type Constraints = HashMap<i32, (StackType, CTStackType)>;

fn pass(
    stmt: &Stmt1,
    cap_pool: &mut CapabilityPool,
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<(Stmt2, HashMap<i32, (StackType, CTStackType)>), Error> {
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
                    None => return Err(TypeErrorEmptyCTStack(*op)),
                    Some(CTType(t)) => {
                        arg_types.push(t);
                        stack_type.push_front(t);
                    }
                    Some(CTCapability(c)) => capabilities.extend(cap_pool.get(c)),
                    Some(x) => {
                        return Err(KindErrorReq(x));
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
                        Some(x) => return Err(KindError(*op, KCapability(None), x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Own() => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => {
                            ct_stack.push(CTCapability(cap_pool.add(vec![Owned(r)])))
                        }
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Read() => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => {
                            ct_stack.push(CTCapability(cap_pool.add(vec![NotOwned(r)])))
                        }
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
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
                                Some(x) => return Err(KindError(*op, KCapability(None), x)),
                                None => return Err(TypeErrorEmptyCTStack(*op)),
                            }
                        }
                        Some(x) => return Err(KindError(*op, KCapability(None), x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Handle() => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTType(type_pool.add(THandle(r)))),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1i32() => ct_stack.push(CTType(type_pool.add(Ti32()))),
                Op1End() => panic!("op-end found during verification"),
                Op1Mut() => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(type_pool.add(TMutable(t)))),
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Tuple(n) => {
                    let mut ts = vec![];
                    for _ in 0..*n {
                        let mb_t = ct_stack.pop();
                        match mb_t {
                            Some(CTType(t)) => ts.push(t),
                            Some(x) => return Err(KindError(*op, KType, x)),
                            None => return Err(TypeErrorEmptyCTStack(*op)),
                        }
                    }
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => {
                            ct_stack.push(CTType(type_pool.add(TTuple(tl_pool.add(ts), r))))
                        }
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Arr() => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(type_pool.add(TArray(t)))),
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
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
                        None => return Err(TypeErrorEmptyExistStack(*op)),
                        Some(id) => {
                            let mb_t = ct_stack.pop();
                            match mb_t {
                                Some(CTType(t)) => {
                                    ct_stack.push(CTType(type_pool.add(TExists(id, t))))
                                }
                                Some(x) => return Err(KindError(*op, KType, x)),
                                None => return Err(TypeErrorEmptyCTStack(*op)),
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
                            Some(x) => return Err(KindError(*op, KType, x)),
                            None => return Err(TypeErrorEmptyCTStack(*op)),
                        }
                    }
                    let mb_c = ct_stack.pop();
                    match mb_c {
                        Some(CTCapability(c)) => {
                            ct_stack.push(CTType(type_pool.add(TFunc(c, tl_pool.add(ts)))))
                        }
                        Some(x) => return Err(KindError(*op, KCapability(None), x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1CTGet(n) => {
                    let l = ct_stack.len();
                    if l == 0 {
                        return Err(TypeErrorEmptyCTStack(*op));
                    }
                    let i = usize::from(*n);
                    if l - 1 < i {
                        return Err(TypeErrorParamOutOfRange(*op));
                    }
                    ct_stack.push(*ct_stack.get(l - i - 1).unwrap());
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
                            _ => return Err(TypeErrorExistentialExpected(tr)),
                        },
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                }
                Op1Get(n) => {
                    let l = stack_type.len();
                    if l == 0 {
                        return Err(TypeErrorEmptyStack(*op));
                    }
                    let i = usize::from(*n);
                    if l - 1 < i {
                        return Err(TypeErrorParamOutOfRange(*op));
                    }
                    stack_type.push_back(*stack_type.get(l - 1 - i).unwrap());
                    out.push(Op2Get(*n))
                }
                Op1Init(n) => {
                    let mb_val = stack_type.pop_back();
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(tpl) {
                            TTuple(ts, r) => match tl_pool.get(*ts).get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(formal) => match mb_val {
                                    None => return Err(TypeErrorEmptyStack(*op)),
                                    Some(actual) => {
                                        if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                            return Err(CapabilityError(
                                                *op,
                                                cap_pool.add(capabilities),
                                            ));
                                        }
                                        if type_pool.get(*formal) == type_pool.get(actual) {
                                            stack_type.push_back(tpl)
                                        } else {
                                            println!("Type error! init is setting a tuple field of the wrong type!");
                                            return Err(TypeErrorInit(*formal, actual));
                                        }
                                    }
                                },
                            },
                            _ => return Err(TypeErrorTupleExpected(*op, tpl)),
                        },
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    out.push(Op2Init(*n))
                }
                Op1Malloc() => {
                    let mb_t = ct_stack.pop();
                    let t = match mb_t {
                        Some(CTType(t)) => t,
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    };
                    let mb_rhandle = stack_type.pop_back();
                    match mb_rhandle {
                        Some(tr) => match type_pool.get(tr) {
                            THandle(r) => {
                                if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                    return Err(CapabilityError(*op, cap_pool.add(capabilities)));
                                }
                            }
                            _ => {
                                println!("Type error! malloc expects a region handle!");
                                return Err(TypeErrorRegionHandleExpected(*op, tr));
                            }
                        },
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    stack_type.push_back(t);
                    out.push(Op2Malloc(4)) // TODO: use actual size in bytes of t
                }
                Op1Proj(n) => {
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(tpl) {
                            TTuple(ts, r) => match tl_pool.get(*ts).get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(t) => {
                                    if capabilities.iter().all(|c| !capable(r, c, &cap_pool)) {
                                        return Err(CapabilityError(
                                            *op,
                                            cap_pool.add(capabilities),
                                        ));
                                    }
                                    stack_type.push_back(*t);
                                }
                            },
                            _ => return Err(TypeErrorTupleExpected(*op, tpl)),
                        },
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    out.push(Op2Proj(*n))
                }
                Op1Call() => {
                    let mb_t = stack_type.pop_back();
                    match mb_t {
                        Some(tr) => {
                            let t = type_pool.get(tr);
                            match t {
                                // TGuess(l) => {
                                //     constraints.insert(*l, (stack_type.clone(), ct_stack.clone()));
                                // }
                                _ => {
                                    let (vars, _caps_ref, _args_ref) = get_func_type_info(t, &type_pool);
                                    let mut assignments = HashMap::new();
                                    for (id, k) in vars {
                                        let mb_ctval = ct_stack.pop();
                                        match mb_ctval {
                                            Some(CTCapability(cr)) => {
                                                match k {
                                                    KCapability(Some(bound)) => {
                                                        if within_bound(&cr, bound, &cap_pool) {
                                                            assignments.insert(id, CTCapability(cr));
                                                        } else {
                                                            return Err(ErrorTodo);
                                                        }
                                                    }
                                                    KCapability(None) => {
                                                        assignments.insert(id, CTCapability(cr));
                                                    }
                                                    _ => return Err(ErrorTodo)
                                                }
                                            }
                                            Some(ctval) => {
                                                if get_kind(&ctval) == *k {
                                                    assignments.insert(id, ctval);
                                                }
                                            }
                                            None => return Err(ErrorTodo)
                                        }
                                    }
                                }
                            }
                        }
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    out.push(Op2Call())
                }
            },
        }
    }
    if exist_stack.len() > 0 {
        return Err(TypeErrorNonEmptyExistStack());
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

fn capable(r: &Region, c: &Capability, cap_pool: &CapabilityPool) -> bool {
    match c {
        Owned(r2) | NotOwned(r2) if *r == *r2 => true,
        CapVarBounded(_, cr) => cap_pool.get(*cr).iter().any(|c| capable(r, c, cap_pool)),
        _ => false,
    }
}

fn get_func_type_info<'a>(func_t: &'a Type, type_pool: &'a TypePool) -> (Vec<(&'a Id, &'a Kind)>, CapabilityRef, TypeListRef) {
    let mut ids: Vec<(&Id, &Kind)> = vec![];
    let mut t = func_t;
    loop {
        match t {
            TForall(id, k, tr) => {
                ids.push((id, k));
                t = type_pool.get(*tr);
            }
            TFunc(cr, tsr) => return (ids, *cr, *tsr),
            _ => panic!("get_func_type_info on non-function type")
        }
    }
}

fn within_bound(cr: &CapabilityRef, boundr: &CapabilityRef, cap_pool: &CapabilityPool) -> bool {
    // let cs = cap_pool.get(*cr);
    // let bound = cap_pool.get(*boundr);
    (cr, boundr, cap_pool);
    true // TODO
}