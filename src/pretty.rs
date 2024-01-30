use crate::header::CTStackVal;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::Kind;
use crate::header::KindContext;
use crate::header::KindContextEntry::*;
use crate::header::OpCode1;
use crate::header::OpCode1::*;
use crate::header::OpCode2;
use crate::header::OpCode2::*;
use crate::header::Region;
use crate::header::Region::*;
use crate::header::Type;
use crate::header::Type::*;
use crate::header::get_kind;

pub fn op2(op: &OpCode2) -> String {
    match op {
        Op2Get(n) => "get ".to_owned() + &n.to_string(),
        Op2Init(n) => "init ".to_owned() + &n.to_string(),
        Op2Malloc(_n) => "malloc".to_owned(),
        Op2Proj(n) => "proj ".to_owned() + &n.to_string(),
        Op2Call => "call".to_owned(),
    }
}

pub fn region(r: Region) -> String {
    match r {
        Heap => "Heap".to_string(),
        RegionVar(id) => "r".to_owned() + &id.1.to_string(),
    }
}

pub fn kind_context(kinds: &KindContext) -> String {
    kinds.iter().map(|entry| 
        match entry {
            KCEntryCapability(id, bound, _) => {
                let prefix = "c".to_owned() + &id.1.to_string();
                if bound.len() != 0 {
                    return prefix + "≤" + &caps(bound)
                } else { 
                    return prefix 
                }
            }
            KCEntryRegion(id, _) => "r".to_owned() + &id.1.to_string(),
            KCEntryType(id, _) => "t".to_owned() + &id.1.to_string()
        }
    ).collect::<Vec<_>>().join(",")
}

pub fn caps(cs: &Vec<Capability>) -> String {
    "{".to_owned() + &cs.iter().map(|c| cap(c.clone())).collect::<Vec<_>>().join(",") + "}"
}

pub fn cap(c: Capability) -> String {
    match c {
        CapVar(id) => "c".to_owned() + &id.1.to_string(),
        Unique(r) => "1".to_owned() + &region(r),
        ReadWrite(r) => "+".to_owned() + &region(r),
    }
}

pub fn types(
    ts: &Vec<Type>,
) -> String {
    ts
        .iter()
        .map(|t| typ(t))
        .collect::<Vec<_>>()
        .join(", ")
}

// fn var(id: &Id, k: &Kind) -> String {
//     match k {
//         KRegion => "r".to_owned() + &id.1.to_string(),
//         KType => "t".to_owned() + &id.1.to_string(),
//         KCapability => "c".to_owned() + &id.1.to_string(),
//     }
// }

pub fn typ(
    t: &Type,
) -> String {
    match t {
        Ti32 => "i32".to_string(),
        THandle(r) => "handle(".to_owned() + &region(*r) + ")",
        TMutable(t) => "mut ".to_owned() + &typ(t),
        TTuple(ts, r) => {
            "(".to_owned() + &types(ts) + ")@" + &region(*r)
        }
        TArray(t) => "[]".to_owned() + &typ(t),
        TVar(id) => "t".to_owned() + &id.1.to_string(),
        TExists(id, t) => {
            "Exists t".to_owned()
                + &id.1.to_string()
                + ". "
                + &typ(t)
        }
        TFunc(kinds, c, ts) => {
            let quantification = 
                if kinds.len() == 0 {
                    "".to_owned()
                } else {
                    "Forall ".to_owned()
                    + &kind_context(kinds)
                    + ". "
                };
            let body = "[".to_owned()
                + &caps(c)
                + "]("
                + &types(ts)
                + ")->0";
            quantification.to_owned() + &body
        }
        TGuess(_) => panic!("type-checking artifact lasted too long"),
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

pub fn op1(op: OpCode1) -> String {
    match op {
        Op1Req => "req".to_owned(),
        Op1Region => "region".to_owned(),
        Op1Heap => "heap".to_owned(),
        Op1Cap => "cap".to_owned(),
        Op1CapLE => "cap_le".to_owned(),
        Op1Own => "own".to_owned(),
        Op1Read => "read".to_owned(),
        Op1Both => "both".to_owned(),
        Op1Handle => "handle".to_owned(),
        Op1i32 => "i32".to_owned(),
        Op1End => "END_FUNC".to_owned(),
        Op1Mut => "mut".to_owned(),
        Op1Tuple(n) => format!("tuple {}", n.to_string()),
        Op1Arr => "arr".to_owned(),
        Op1All => "all".to_owned(),
        Op1Some => "some".to_owned(),
        Op1Emos => "emos".to_owned(),
        Op1Func(n) => format!("func {}", n.to_string()),
        Op1CTGet(n) => format!("ct_get {}", n.to_string()),
        Op1CTPop => "ct_pop".to_owned(),
        Op1Unpack => "unpack".to_owned(),
        Op1Get(n) => format!("get {}", n.to_string()),
        Op1Init(n) => format!("init {}", n.to_string()),
        Op1Malloc => "malloc".to_owned(),
        Op1Proj(n) => format!("proj {}", n.to_string()),
        Op1Call => "call".to_owned()
    }
}

pub fn kind(k: Kind) -> String {
    (match k {
        Kind::KCapability => "capability",
        Kind::KRegion => "region",
        Kind::KType => "type",
    })
    .to_owned()
}