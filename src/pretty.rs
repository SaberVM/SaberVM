use crate::header::CTStackVal;
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
use crate::header::get_kind;

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
        // TGuess(_) => panic!("type-checking artifact lasted too long"),
    }
}


pub fn get_kind_str(ctval: &CTStackVal) -> String {
    kind(get_kind(ctval))
}

pub fn op_u8(byte: u8) -> String {
    (match byte {
        0x00 => "req",
        0x01 => "region",
        0x02 => "heap",
        0x03 => "cap",
        0x04 => "cap_le",
        0x05 => "own",
        0x06 => "read",
        0x07 => "both",
        0x08 => "handle",
        0x09 => "i32",
        0x0A => "END_FUNC",
        0x0B => "mut",
        0x0C => "tuple",
        0x0D => "arr",
        0x0E => "all",
        0x0F => "some",
        0x10 => "emos",
        0x11 => "func",
        0x12 => "ct_get",
        0x13 => "ct_pop",
        0x14 => "unpack",
        0x15 => "get",
        0x16 => "init",
        0x17 => "malloc",
        0x18 => "proj",
        0x19 => "call",
        _ => panic!("unknown opcode {}", byte),
    })
    .to_owned()
}

pub fn kind(k: Kind) -> String {
    (match k {
        Kind::KCapability(_) => "capability",
        Kind::KRegion => "region",
        Kind::KType => "type",
    })
    .to_owned()
}