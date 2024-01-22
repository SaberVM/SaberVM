use std::collections::HashMap;
use std::collections::VecDeque;

use crate::header::CTStackVal;
use crate::header::CTStackVal::*;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::Error;
use crate::header::Error::*;
use crate::header::Id;
use crate::header::Kind::*;
use crate::header::KindContext;
use crate::header::KindContextEntry::*;
use crate::header::OpCode1::*;
use crate::header::OpCode2;
use crate::header::OpCode2::*;
use crate::header::Region;
use crate::header::Region::*;
use crate::header::Stmt1;
use crate::header::Stmt1::*;
use crate::header::Stmt2;
use crate::header::Stmt2::*;
use crate::header::Type::*;
use crate::header::TypeListPool;
use crate::header::TypePool;
use crate::header::TypeRef;

pub fn go(
    stmts: Vec<Stmt1>,
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<Vec<Stmt2>, Error> {
    let mut out: Vec<Stmt2> = vec![];
    let stmts2: Vec<(Stmt2, Constraints)> = stmts
        .iter()
        .map(|stmt| pass(stmt, type_pool, tl_pool))
        .collect::<Result<Vec<_>, Error>>()
        .unwrap();
    // let mut types: HashMap<i32, Type> = HashMap::new();
    // for pair in &stmts2 {
    //     let (Func2(label, t, _), _) = pair;
    //     types.insert(*label, *t);
    // }
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
    type_pool: &mut TypePool,
    tl_pool: &mut TypeListPool,
) -> Result<(Stmt2, HashMap<i32, (StackType, CTStackType)>), Error> {
    let mut ct_stack: CTStackType = vec![];
    let Func1(label, ops) = stmt;
    let mut iter = ops.iter();
    let mut arg_types: Vec<TypeRef> = vec![];
    let mut stack_type: StackType = VecDeque::from([]);
    let mut rvars: Vec<Id> = vec![];
    let mut capabilities_needed: Vec<Capability> = vec![];
    let mut capability_bounds: HashMap<Id, Vec<Capability>> = HashMap::new();
    let mut tvars: Vec<Id> = vec![];
    let mut kind_context: KindContext = vec![];
    let mut exist_stack: Vec<Id> = vec![];
    let mut op2s: Vec<OpCode2> = vec![];
    let mut fresh_id = 0;
    let mut constraints = HashMap::new();
    loop {
        match iter.next() {
            None => break,
            Some(op) => match op {
                Op1Req => match ct_stack.pop() {
                    None => return Err(TypeErrorEmptyCTStack(*op)),
                    Some(CTType(t)) => {
                        arg_types.push(t);
                        stack_type.push_front(t);
                    }
                    Some(CTCapability(cs)) => capabilities_needed.extend(cs),
                    Some(x) => {
                        return Err(KindErrorReq(x));
                    }
                },
                Op1Region => {
                    let id = Id(*label, fresh_id);
                    ct_stack.push(CTRegion(RegionVar(id)));
                    rvars.push(id);
                    kind_context.push(KCEntryRegion(id));
                    fresh_id += 1;
                }
                Op1Heap => ct_stack.push(CTRegion(Heap)),
                Op1Cap => {
                    let id = Id(*label, fresh_id);
                    let var = CapVar(id);
                    let cap = vec![var];
                    capability_bounds.insert(id, vec![]);
                    kind_context.push(KCEntryCapability(id, vec![]));
                    ct_stack.push(CTCapability(cap));
                    fresh_id += 1;
                }
                Op1CapLE => {
                    let mb_bound = ct_stack.pop();
                    match mb_bound {
                        Some(CTCapability(bound)) => {
                            let id = Id(*label, fresh_id);
                            let var = CapVar(id);
                            ct_stack.push(CTCapability(vec![var]));
                            capability_bounds.insert(id, bound.clone());
                            kind_context.push(KCEntryCapability(id, bound));
                            fresh_id += 1;
                        }
                        Some(x) => return Err(KindError(*op, KCapability, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Own => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTCapability(vec![Owned(r)])),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Read => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTCapability(vec![NotOwned(r)])),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Both => {
                    let mb_c1 = ct_stack.pop();
                    match mb_c1 {
                        Some(CTCapability(c1)) => {
                            let mb_c2 = ct_stack.pop();
                            match mb_c2 {
                                Some(CTCapability(c2)) => {
                                    ct_stack.push(CTCapability([&c1[..], &c2[..]].concat()))
                                }
                                Some(x) => return Err(KindError(*op, KCapability, x)),
                                None => return Err(TypeErrorEmptyCTStack(*op)),
                            }
                        }
                        Some(x) => return Err(KindError(*op, KCapability, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Handle => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTType(type_pool.add(THandle(r)))),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1i32 => ct_stack.push(CTType(type_pool.add(Ti32))),
                Op1End => panic!("op-end found during verification"),
                Op1Mut => {
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
                Op1Arr => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(type_pool.add(TArray(t)))),
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1All => {
                    let id = Id(*label, fresh_id);
                    ct_stack.push(CTType(type_pool.add(TVar(id))));
                    tvars.push(id);
                    kind_context.push(KCEntryType(id));
                    fresh_id += 1
                }
                Op1Some => {
                    let id = Id(*label, fresh_id);
                    ct_stack.push(CTType(type_pool.add(TVar(id))));
                    exist_stack.push(id);
                    fresh_id += 1;
                }
                Op1Emos => {
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
                            ct_stack.push(CTType(type_pool.add(TFunc(vec![], c, tl_pool.add(ts)))))
                        }
                        Some(x) => return Err(KindError(*op, KCapability, x)),
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
                    ct_stack.push(ct_stack.get(l - i - 1).unwrap().clone());
                }
                Op1CTPop => {
                    ct_stack.pop();
                }
                Op1Unpack => {
                    let mb_ex = stack_type.pop_back();
                    match mb_ex {
                        Some(tr) => match type_pool.get(&tr) {
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
                    op2s.push(Op2Get(*n))
                }
                Op1Init(n) => {
                    let mb_val = stack_type.pop_back();
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(&tpl) {
                            TTuple(ts, r) => match tl_pool.get(ts).get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(formal) => match mb_val {
                                    None => return Err(TypeErrorEmptyStack(*op)),
                                    Some(actual) => {
                                        if capabilities_needed
                                            .iter()
                                            .all(|c| !capable_not_owned(r, c, &capability_bounds))
                                        {
                                            return Err(CapabilityError(*op, capabilities_needed));
                                        }
                                        if type_pool.get(formal) == type_pool.get(&actual) {
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
                    op2s.push(Op2Init(*n))
                }
                Op1Malloc => {
                    let mb_t = ct_stack.pop();
                    let t = match mb_t {
                        Some(CTType(t)) => t,
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    };
                    let mb_rhandle = stack_type.pop_back();
                    match mb_rhandle {
                        Some(tr) => match type_pool.get(&tr) {
                            THandle(r) => {
                                if capabilities_needed
                                    .iter()
                                    .all(|c| !capable_not_owned(r, c, &capability_bounds))
                                {
                                    return Err(CapabilityError(*op, capabilities_needed));
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
                    op2s.push(Op2Malloc(4)) // TODO: use actual size in bytes of t
                }
                Op1Proj(n) => {
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match type_pool.get(&tpl) {
                            TTuple(ts, r) => match tl_pool.get(ts).get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(t) => {
                                    if capabilities_needed
                                        .iter()
                                        .all(|c| !capable_not_owned(r, c, &capability_bounds))
                                    {
                                        return Err(CapabilityError(*op, capabilities_needed));
                                    }
                                    stack_type.push_back(*t);
                                }
                            },
                            _ => return Err(TypeErrorTupleExpected(*op, tpl)),
                        },
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    op2s.push(Op2Proj(*n))
                }
                Op1Call => {
                    let mb_t = stack_type.pop_back();
                    match mb_t {
                        Some(tr) => {
                            let t = type_pool.get(&tr).clone();
                            match t {
                                TGuess(l) => {
                                    constraints.insert(l, (stack_type.clone(), ct_stack.clone()));
                                }
                                TFunc(quantified, caps_needed, args_ref) => {
                                    let mut instantiation = ct_stack.iter().take(quantified.len());
                                    let caps_present = &capabilities_needed;
                                    let arg_ts_needed = tl_pool.get(&args_ref);
                                    let arg_ts_present =
                                        stack_type.iter().take(arg_ts_needed.len());
                                    // check that the quantified vars can be instantiated by the instantiation
                                    let mut cap_assignments: HashMap<Id, &Vec<Capability>> = HashMap::new();
                                    let mut rgn_assignments: HashMap<Id, &Region> = HashMap::new();
                                    let mut type_assignments: HashMap<Id, &TypeRef> = HashMap::new();
                                    if instantiation.len() != quantified.len() {
                                        return Err(ErrorTodo);
                                    }
                                    for entry in quantified {
                                        let actual = instantiation.next().unwrap();
                                        match entry {
                                            KCEntryCapability(id, bound) => 
                                                match actual {
                                                    CTCapability(c) => {
                                                        // check that the instantiated capability is more restrictive than the formal one, or equally restrictive
                                                        if caps_satisfy_caps(c, &bound, &capability_bounds) {
                                                            cap_assignments.insert(id, c);
                                                        } else {
                                                            return Err(ErrorTodo)
                                                        }
                                                    }
                                                    _ => return Err(ErrorTodo) // kind error- instantiated cap var with a noncap
                                                }
                                            KCEntryRegion(id) => 
                                                match actual {
                                                    CTRegion(r) => {
                                                        rgn_assignments.insert(id, r);
                                                    }
                                                    _ => return Err(ErrorTodo) // kind error- instantiated rgn var with a nonrgn
                                                }
                                            KCEntryType(id) =>
                                                match actual {
                                                    CTType(tr) => {
                                                        type_assignments.insert(id, tr);
                                                    }
                                                    _ => return Err(ErrorTodo) // kind error- instantiated type var with a nontype
                                                }
                                        }
                                    }
                                    // then check that the actual capabilities satisfy the needed capabilities
                                    if !caps_satisfy_caps_with_subs(caps_present, &caps_needed, &capability_bounds, &cap_assignments, &rgn_assignments) {
                                        return Err(ErrorTodo);
                                    }
                                    // then check that the parameter types match the top of the stack type
                                }
                                _ => return Err(TypeErrorFunctionExpected(*op, tr))
                            }
                        }
                        None => return Err(TypeErrorEmptyStack(*op)),
                    }
                    op2s.push(Op2Call)
                }
            },
        }
    }
    if exist_stack.len() > 0 {
        return Err(TypeErrorNonEmptyExistStack);
    }
    let t = type_pool.add(TFunc(kind_context, capabilities_needed, tl_pool.add(arg_types)));
    Ok((Func2(*label, t, op2s), constraints))
}

fn capable_not_owned(r: &Region, c: &Capability, cap_bounds: &HashMap<Id, Vec<Capability>>) -> bool {
    match c {
        Owned(r2) | NotOwned(r2) if *r == *r2 => true,
        CapVar(id) => {
            let cs = cap_bounds.get(id).unwrap();
            cs.iter().any(|c| capable_not_owned(r, c, cap_bounds))
        }
        _ => false,
    }
}

fn cap_subtype(c1: &Capability, c2: &Capability, cap_bounds: &HashMap<Id, Vec<Capability>>) -> bool {
    match (c1, c2) {
        (Owned(r1), Owned(r2)) if r1 == r2 => true,
        (Owned(r1), NotOwned(r2)) if r1 == r2 => true,
        (NotOwned(r1), NotOwned(r2)) if r1 == r2 => true,
        (NotOwned(_), Owned(_)) => false,
        (CapVar(id), c2) => {
            let bound = cap_bounds.get(id).unwrap();
            caps_satisfy_cap(bound, c2, cap_bounds)
        }
        _ => false, // capability variables that have lasted this long are uninformative
    }
}

fn caps_satisfy_cap(
    caps: &Vec<Capability>,
    cap: &Capability,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> bool {
    caps.iter().any(|c_p| cap_subtype(c_p, cap, cap_bounds))
}

fn caps_satisfy_caps(
    caps1: &Vec<Capability>,
    caps2: &Vec<Capability>,
    cap_bounds: &HashMap<Id, Vec<Capability>>
) -> bool {
    caps2.iter().all(|c| caps_satisfy_cap(caps1, c, cap_bounds))
}

fn caps_satisfy_caps_with_subs(
    caps1: &Vec<Capability>,
    caps2: &Vec<Capability>,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
    cap_assignments: &HashMap<Id, &Vec<Capability>>,
    rgn_assignments: &HashMap<Id, &Region>
) -> bool {
    // this function seems like it could be a spot for optimization later
    for c in caps2 {
        match c {
            CapVar(id) => 
                match cap_assignments.get(id) {
                    Some(cs) => 
                        if !caps_satisfy_caps(caps1, cs, cap_bounds) {
                            return false;
                        }
                    None => 
                        if !caps_satisfy_cap(caps1, c, cap_bounds) {
                            return false;
                        }
                }
            Owned(RegionVar(id)) => 
                match rgn_assignments.get(id) {
                    Some(r) => 
                        if !caps_satisfy_cap(caps1, &Owned(**r), cap_bounds) {
                            return false;
                        }
                    None => 
                        if !caps_satisfy_cap(caps1, c, cap_bounds) {
                            return false;
                        }
                }
            NotOwned(RegionVar(id)) => 
                match rgn_assignments.get(id) {
                    Some(r) =>
                        if !caps_satisfy_cap(caps1, &NotOwned(**r), cap_bounds) {
                            return false;
                        }
                    None =>
                        if !caps_satisfy_cap(caps1, c, cap_bounds) {
                            return false;
                        }
                }
            _ => // there are no substitutions to be made for this needed capability
                if !caps_satisfy_cap(caps1, c, cap_bounds) {
                    return false;
                }
        }
    }
    // every needed capability (under substitution) was satisfied if we made it this far
    return true;
}