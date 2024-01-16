use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::CapabilityPool;
use crate::header::Id;
use crate::header::Kind;
use crate::header::Kind::*;
use crate::header::OpCode2;
use crate::header::OpCode2::*;
use crate::header::Region;
use crate::header::Region::*;
use crate::header::Type;
use crate::header::Type::*;
use crate::header::TypeListPool;
use crate::header::TypeListRef;
use crate::header::TypePool;

pub fn op2(op: &OpCode2) -> String {
    match op {
        Op2Get(n) => "get ".to_owned() + &n.to_string(),
        Op2Init(n) => "init ".to_owned() + &n.to_string(),
        Op2Malloc(_n) => "malloc".to_owned(),
        Op2Proj(n) => "proj ".to_owned() + &n.to_string(),
        Op2Clean(n, _m) => "clean ".to_owned() + &n.to_string(),
        Op2Call() => "call".to_owned(),
    }
}

pub fn region(r: Region) -> String {
    match r {
        Heap() => "Heap".to_string(),
        RegionVar(id) => "r".to_owned() + &id.1.to_string(),
    }
}

pub fn caps(cs: &Vec<Capability>) -> String {
    "{".to_owned() + &cs.iter().map(|c| cap(*c)).collect::<Vec<_>>().join(",") + "}"
}

pub fn cap(c: Capability) -> String {
    match c {
        CapVar(id) => "c".to_owned() + &id.1.to_string(),
        CapVarBounded(id, _cr) => "c".to_owned() + &id.1.to_string(),
        Owned(r) => "1".to_owned() + &region(r),
        NotOwned(r) => "+".to_owned() + &region(r),
    }
}

pub fn types(
    ts: &TypeListRef,
    type_pool: &TypePool,
    tl_pool: &TypeListPool,
    cap_pool: &CapabilityPool,
) -> String {
    tl_pool
        .get(*ts)
        .iter()
        .map(|tr| typ(type_pool.get(*tr), type_pool, tl_pool, cap_pool))
        .collect::<Vec<_>>()
        .join(", ")
}

fn var(id: &Id, k: Kind, cap_pool: &CapabilityPool) -> String {
    match k {
        KRegion => "r".to_owned() + &id.1.to_string(),
        KType => "t".to_owned() + &id.1.to_string(),
        KCapability(None) => "c".to_owned() + &id.1.to_string(),
        KCapability(Some(c)) => "c".to_owned() + &id.1.to_string() + "≤" + &caps(cap_pool.get(c)),
    }
}

pub fn typ(
    t: &Type,
    type_pool: &TypePool,
    tl_pool: &TypeListPool,
    cap_pool: &CapabilityPool,
) -> String {
    match t {
        Ti32() => "i32".to_string(),
        THandle(r) => "handle(".to_owned() + &region(*r) + ")",
        TMutable(tr) => "mut ".to_owned() + &typ(type_pool.get(*tr), type_pool, tl_pool, cap_pool),
        TTuple(ts, r) => {
            "(".to_owned() + &types(ts, type_pool, tl_pool, cap_pool) + ")@" + &region(*r)
        }
        TArray(tr) => "[]".to_owned() + &typ(type_pool.get(*tr), type_pool, tl_pool, cap_pool),
        TVar(id) => "t".to_owned() + &id.1.to_string(),
        TForall(id, k, tr) => {
            "Forall ".to_owned()
                + &var(&id, *k, cap_pool)
                + ". "
                + &typ(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        TExists(id, tr) => {
            "Exists t".to_owned()
                + &id.1.to_string()
                + ". "
                + &typ(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        TFunc(c, ts) => {
            "[".to_owned()
                + &caps(cap_pool.get(*c))
                + "]("
                + &types(ts, type_pool, tl_pool, cap_pool)
                + ")->0"
        }
        TGuess(_) => panic!("type-checking artifact lasted too long"),
    }
}
