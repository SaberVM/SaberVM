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
use crate::header::Type;
use crate::header::Type::*;
use crate::pretty;

pub fn go(stmts: Vec<Stmt1>) -> Result<Vec<Stmt2>, Error> {
    let mut out: Vec<Stmt2> = vec![];
    let stmts2: Vec<(Stmt2, Constraints)> = stmts
        .iter()
        .map(|stmt| pass(stmt))
        .collect::<Result<Vec<_>, Error>>()?;
    let mut constraints: Constraints = HashMap::new();
    for pair in stmts2 {
        // TODO: check that the expected types of global functions are their actual types
        let (stmt, c) = pair;
        constraints.extend(c);
        out.push(stmt);
    }
    return Ok(out);
}

type StackType = VecDeque<Type>;
type CTStackType = Vec<CTStackVal>;
type Constraints = HashMap<i32, (StackType, CTStackType)>;

fn pass(stmt: &Stmt1) -> Result<(Stmt2, HashMap<i32, (StackType, CTStackType)>), Error> {
    let mut ct_stack: CTStackType = vec![];
    let Func1(label, ops) = stmt;
    let mut iter = ops.iter();
    let mut arg_types: Vec<Type> = vec![];
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
                        arg_types.push(t.clone());
                        stack_type.push_front(t);
                    }
                    Some(CTCapability(cs)) => capabilities_needed.extend(cs),
                    Some(x) => {
                        return Err(KindErrorReq(x));
                    }
                },
                Op1Region => {
                    let id = Id(*label, fresh_id);
                    let r = RegionVar(id);
                    ct_stack.push(CTRegion(r));
                    rvars.push(id);
                    kind_context.push(KCEntryRegion(id, r));
                    fresh_id += 1;
                }
                Op1Heap => ct_stack.push(CTRegion(Heap)),
                Op1Cap => {
                    let id = Id(*label, fresh_id);
                    let var = CapVar(id);
                    let cap = vec![var.clone()];
                    capability_bounds.insert(id, vec![]);
                    kind_context.push(KCEntryCapability(id, vec![], var));
                    ct_stack.push(CTCapability(cap));
                    fresh_id += 1;
                }
                Op1CapLE => {
                    let mb_bound = ct_stack.pop();
                    match mb_bound {
                        Some(CTCapability(bound)) => {
                            let id = Id(*label, fresh_id);
                            let var = CapVar(id);
                            ct_stack.push(CTCapability(vec![var.clone()]));
                            capability_bounds.insert(id, bound.clone());
                            kind_context.push(KCEntryCapability(id, bound, var));
                            fresh_id += 1;
                        }
                        Some(x) => return Err(KindError(*op, KCapability, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Own => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTCapability(vec![Unique(r)])),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Read => {
                    let mb_r = ct_stack.pop();
                    match mb_r {
                        Some(CTRegion(r)) => ct_stack.push(CTCapability(vec![ReadWrite(r)])),
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
                        Some(CTRegion(r)) => ct_stack.push(CTType(THandle(r))),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1i32 => ct_stack.push(CTType(Ti32)),
                Op1End => panic!("op-end found during verification"),
                Op1Mut => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(TMutable(Box::new(t)))),
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
                        Some(CTRegion(r)) => ct_stack.push(CTType(TTuple(ts, r))),
                        Some(x) => return Err(KindError(*op, KRegion, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1Arr => {
                    let mb_t = ct_stack.pop();
                    match mb_t {
                        Some(CTType(t)) => ct_stack.push(CTType(TArray(Box::new(t)))),
                        Some(x) => return Err(KindError(*op, KType, x)),
                        None => return Err(TypeErrorEmptyCTStack(*op)),
                    }
                }
                Op1All => {
                    let id = Id(*label, fresh_id);
                    let t = TVar(id);
                    ct_stack.push(CTType(t.clone()));
                    tvars.push(id);
                    kind_context.push(KCEntryType(id, t));
                    fresh_id += 1
                }
                Op1Some => {
                    let id = Id(*label, fresh_id);
                    ct_stack.push(CTType(TVar(id)));
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
                                Some(CTType(t)) => ct_stack.push(CTType(TExists(id, Box::new(t)))),
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
                        Some(CTCapability(c)) => ct_stack.push(CTType(TFunc(vec![], c, ts))),
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
                        Some(t) => match t {
                            TExists(_id, t) => {
                                stack_type.push_back(*t) // simply remove the quantifier, unbinding its variable
                            }
                            _ => return Err(TypeErrorExistentialExpected(t)),
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
                    stack_type.push_back(stack_type.get(l - 1 - i).unwrap().clone());
                    op2s.push(Op2Get(*n))
                }
                Op1Init(n) => {
                    let mb_val = stack_type.pop_back();
                    let mb_tpl = stack_type.pop_back();
                    match mb_tpl {
                        Some(tpl) => match tpl.clone() {
                            TTuple(ts, r) => match ts.get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(formal) => match mb_val {
                                    None => return Err(TypeErrorEmptyStack(*op)),
                                    Some(actual) => {
                                        if capabilities_needed
                                            .iter()
                                            .all(|c| !capable_read_write(&r, c, &capability_bounds))
                                        {
                                            return Err(CapabilityError(*op, capabilities_needed));
                                        }
                                        if formal == &actual {
                                            stack_type.push_back(tpl)
                                        } else {
                                            println!("Type error! init is setting a tuple field of the wrong type!");
                                            return Err(TypeErrorInit(formal.clone(), actual));
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
                        Some(t) => match t {
                            THandle(r) => {
                                if capabilities_needed
                                    .iter()
                                    .all(|c| !capable_read_write(&r, c, &capability_bounds))
                                {
                                    return Err(CapabilityError(*op, capabilities_needed));
                                }
                            }
                            _ => {
                                println!("Type error! malloc expects a region handle!");
                                return Err(TypeErrorRegionHandleExpected(*op, t));
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
                        Some(tpl) => match tpl {
                            TTuple(ts, r) => match ts.get(usize::from(*n)) {
                                None => return Err(TypeErrorParamOutOfRange(*op)),
                                Some(t) => {
                                    if capabilities_needed
                                        .iter()
                                        .all(|c| !capable_read_write(&r, c, &capability_bounds))
                                    {
                                        return Err(CapabilityError(*op, capabilities_needed));
                                    }
                                    stack_type.push_back(t.clone());
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
                        Some(t) => match t {
                            TGuess(l) => {
                                constraints.insert(l, (stack_type.clone(), ct_stack.clone()));
                            }
                            TFunc(quantified, caps_needed, args) => {
                                let mut instantiation = vec![];
                                for _ in 0..quantified.len() {
                                    match ct_stack.pop() {
                                        Some(ctval) => instantiation.push(ctval),
                                        None => return Err(ErrorTodo("not enough arguments on the compile-time stack to call the function".to_owned()))
                                    }
                                }
                                let caps_present = &capabilities_needed;
                                let arg_ts_needed = &args;
                                let mut arg_ts_present = vec![];
                                for _ in 0..arg_ts_needed.len() {
                                    match stack_type.pop_back() {
                                        Some(t) => arg_ts_present.push(t.clone()),
                                        None => {
                                            return Err(ErrorTodo("not enough arguments on the stack to call the function!".to_owned()));
                                        }
                                    }
                                }
                                let (rgn_assignments, cap_assignments, type_assignments) =
                                    instantiate(instantiation, quantified, &capability_bounds)?;
                                let caps_needed_subbed = substitute_c(&caps_needed, &rgn_assignments, &cap_assignments);
                                let caps_are_sufficient = caps_satisfy_caps(caps_present, &caps_needed_subbed, &capability_bounds);
                                if !caps_are_sufficient {
                                    dbg!(pretty::caps(&caps_present));
                                    dbg!(pretty::caps(&caps_needed_subbed));
                                    return Err(ErrorTodo(
                                        "insufficient capabilities for call".to_owned(),
                                    ));
                                }
                                let types_match = arg_ts_present.iter().zip(arg_ts_needed.iter()).all(|(t1, t2)| {
                                    type_eq(
                                        t1,
                                        &substitute_t(
                                            t2,
                                            &type_assignments,
                                            &rgn_assignments,
                                            &cap_assignments,
                                        ),
                                    )
                                });
                                if !types_match {
                                    let arg_ts_present = arg_ts_present
                                        .iter()
                                        .map(|t| pretty::typ(&t))
                                        .collect::<Vec<_>>();
                                    dbg!(&arg_ts_present);
                                    let arg_ts_needed = arg_ts_needed
                                        .iter()
                                        .map(|t| substitute_t(t, &type_assignments, &rgn_assignments, &cap_assignments))
                                        .map(|t| pretty::typ(&t))
                                        .collect::<Vec<_>>();
                                    dbg!(&arg_ts_needed);
                                    return Err(ErrorTodo("incorrect types for call".to_owned()));
                                }
                            }
                            _ => return Err(TypeErrorFunctionExpected(*op, t)),
                        },
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
    let t = TFunc(kind_context, capabilities_needed, arg_types);
    Ok((Func2(*label, t, op2s), constraints))
}

fn capable_read_write(
    r: &Region,
    c: &Capability,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> bool {
    match c {
        Unique(r2) | ReadWrite(r2) if *r == *r2 => true,
        CapVar(id) => {
            let cs = cap_bounds.get(id).unwrap();
            cs.iter().any(|c| capable_read_write(r, c, cap_bounds))
        }
        _ => false,
    }
}

fn instantiate(
    ct_args: Vec<CTStackVal>,
    quantified: KindContext,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> Result<
    (
        HashMap<Id, Region>,
        HashMap<Id, Vec<Capability>>,
        HashMap<Id, Type>,
    ),
    Error,
> {
    let mut ct_args = ct_args.iter();
    let mut cap_assignments: HashMap<Id, Vec<Capability>> = HashMap::new();
    let mut rgn_assignments: HashMap<Id, Region> = HashMap::new();
    let mut type_assignments: HashMap<Id, Type> = HashMap::new();
    if ct_args.len() != quantified.len() {
        return Err(ErrorTodo(
            "not enough ct stack vals to instantiate call".to_owned(),
        ));
    }

    for entry in quantified {
        let actual = ct_args.next().unwrap();
        match entry {
            KCEntryCapability(id, bound, _) => match actual {
                CTCapability(c) => {
                    // check that the instantiated capability is more restrictive than the formal one, or equally restrictive
                    if caps_satisfy_caps(c, &bound, &cap_bounds) {
                        cap_assignments.insert(id, c.to_vec());
                    } else {
                        return Err(ErrorTodo(
                            "insufficient caps for capvar instantiation".to_owned(),
                        ));
                    }
                }
                _ => {
                    return Err(ErrorTodo(
                        "kind error, instantiated a capvar with a noncap".to_owned(),
                    ))
                }
            },
            KCEntryRegion(id, _) => match actual {
                CTRegion(r) => {
                    rgn_assignments.insert(id, *r);
                }
                _ => {
                    return Err(ErrorTodo(
                        "kind error, instantiated a rgnvar with a nonrgn".to_owned(),
                    ))
                } // kind error- instantiated rgn var with a nonrgn
            },
            KCEntryType(id, _) => match actual {
                CTType(t) => {
                    type_assignments.insert(id, t.clone());
                }
                _ => {
                    return Err(ErrorTodo(
                        "kind error, instantiated a tvar with a nontype".to_owned(),
                    ))
                } // kind error- instantiated type var with a nontype
            },
        }
    }
    // then check that the parameter types match the top of the stack type
    Ok((rgn_assignments, cap_assignments, type_assignments))
}

fn cap_subtype(
    c1: &Capability,
    c2: &Capability,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> bool {
    match (c1, c2) {
        (Unique(r1), Unique(r2)) if r1 == r2 => true,
        (Unique(r1), ReadWrite(r2)) if r1 == r2 => true,
        (ReadWrite(r1), ReadWrite(r2)) if r1 == r2 => true,
        (ReadWrite(_), Unique(_)) => false,
        (CapVar(id1), CapVar(id2)) if id1 == id2 => true,
        (CapVar(id), c2) => {
            let bound = cap_bounds.get(id).unwrap();
            caps_satisfy_cap(bound, c2, cap_bounds)
        }
        _ => false, // capability variables that have lasted this long are uninformative
    }
}

fn caps_satisfy_cap(
    caps: &[Capability],
    cap: &Capability,
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> bool {
    caps.iter().any(|c_p| cap_subtype(c_p, cap, cap_bounds))
}

fn caps_satisfy_caps(
    caps1: &[Capability],
    caps2: &[Capability],
    cap_bounds: &HashMap<Id, Vec<Capability>>,
) -> bool {
    caps2.iter().all(|c| caps_satisfy_cap(caps1, c, cap_bounds))
}

fn substitute_t(
    t: &Type,
    tsubs: &HashMap<Id, Type>,
    rsubs: &HashMap<Id, Region>,
    csubs: &HashMap<Id, Vec<Capability>>,
) -> Type {
    match t {
        Ti32 => Ti32,
        THandle(r) => THandle(substitute_r(r, rsubs)),
        TMutable(t) => TMutable(Box::new(substitute_t(t, tsubs, rsubs, csubs))),
        TTuple(ts, r) => TTuple(
            ts.iter()
                .map(|t| substitute_t(t, tsubs, rsubs, csubs))
                .collect(),
            substitute_r(r, rsubs),
        ),
        TArray(t) => TArray(Box::new(substitute_t(t, tsubs, rsubs, csubs))),
        TVar(id) => match tsubs.get(id) {
            Some(new) => new.clone(),
            None => TVar(*id),
        },
        TFunc(kind_context, caps, args) => TFunc(
            kind_context.clone(),
            substitute_c(caps, rsubs, csubs),
            args.iter()
                .map(|t| substitute_t(t, tsubs, rsubs, csubs))
                .collect(),
        ),
        TExists(x, t) => TExists(*x, Box::new(substitute_t(t, tsubs, rsubs, csubs))),
        TGuess(label) => TGuess(*label),
    }
}

fn substitute_r(r: &Region, rsubs: &HashMap<Id, Region>) -> Region {
    match r {
        Heap => Heap,
        RegionVar(id) => match rsubs.get(id) {
            Some(new) => new.clone(),
            None => RegionVar(*id),
        },
    }
}

fn substitute_c(
    caps: &Vec<Capability>,
    rsubs: &HashMap<Id, Region>,
    csubs: &HashMap<Id, Vec<Capability>>,
) -> Vec<Capability> {
    let mut out = vec![];
    for c in caps {
        match c {
            Unique(r) => out.push(Unique(substitute_r(r, rsubs))),
            ReadWrite(r) => out.push(ReadWrite(substitute_r(r, rsubs))),
            CapVar(id) => match csubs.get(id) {
                Some(new) => out.extend(new.clone()),
                None => out.push(CapVar(*id)),
            },
        }
    }
    return out;
}

fn type_eq(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Ti32, Ti32) => true,
        (THandle(r1), THandle(r2)) => r1 == r2,
        (TMutable(t1), TMutable(t2)) => type_eq(t1, t2),
        (TTuple(ts1, r1), TTuple(ts2, r2)) => {
            r1 == r2 && ts1.len() == ts2.len() && {
                let mut ts2 = ts2.iter();
                for t1 in ts1 {
                    let t2 = ts2.next().unwrap();
                    if !type_eq(t1, t2) {
                        return false;
                    }
                }
                return true;
            }
        }
        (TArray(t1), TArray(t2)) => type_eq(t1, t2),
        (TVar(id1), TVar(id2)) => id1 == id2,
        (TFunc(kind_context1, _caps1, ts1), TFunc(kind_context2, _caps2, ts2)) => {
            if kind_context1.len() != kind_context2.len() {
                return false;
            }
            let mut kind_context2_iter = kind_context2.iter();
            let mut cap_assignments = HashMap::new();
            let mut rgn_assignments = HashMap::new();
            let mut type_assignments = HashMap::new();
            for entry1 in kind_context1 {
                let entry2 = kind_context2_iter.next().unwrap();
                match (entry1, entry2) {
                    (KCEntryCapability(id1, _bound1, _), KCEntryCapability(id2, _bound2, _)) => {
                        // TODO: check capability equivalence of bounds
                        cap_assignments.insert(*id2, vec![CapVar(*id1)]);
                    }
                    (KCEntryRegion(id1, _), KCEntryRegion(id2, _)) => {
                        rgn_assignments.insert(*id2, RegionVar(*id1));
                    }
                    (KCEntryType(id1, _), KCEntryType(id2, _)) => {
                        type_assignments.insert(*id2, TVar(*id1));
                    }
                    _ => return false,
                }
            }
            let types_match = ts1.iter().zip(ts2.iter()).all(|(t1, t2)| {
                type_eq(
                    t1,
                    &substitute_t(t2, &type_assignments, &rgn_assignments, &cap_assignments),
                )
            });
            if !types_match {
                return false;
            }
            // TODO: check capability equivalence
            true
        }
        (TExists(id1, t1), TExists(id2, t2)) => {
            let mut sub = HashMap::new();
            sub.insert(*id2, TVar(*id1));
            let t2_subbed = substitute_t(t2, &sub, &HashMap::new(), &HashMap::new());
            type_eq(t1, &t2_subbed)
        }
        (TGuess(label1), TGuess(label2)) => label1 == label2,
        (_, _) => false,
    }
}
