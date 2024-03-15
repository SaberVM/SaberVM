/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::get_kind;
use crate::header::CTStackVal;
use crate::header::Capability;
use crate::header::Capability::*;
use crate::header::Kind;
use crate::header::KindContext;
use crate::header::KindContextEntry::*;
use crate::header::Region;
use crate::header::Region::*;
use crate::header::Repr;
use crate::header::Type;
use crate::header::Type::*;
use crate::header::UnverifiedOpcode;
use crate::header::UnverifiedOpcode::*;
// use crate::header::VerifiedOpcode;

/// Get a pretty string representation of a verified opcode.
// pub fn verified_op(op: &VerifiedOpcode) -> String {
//     match op {
//         VerifiedOpcode::GetOp(offset, size) => format!("get {} {}", offset, size),
//         VerifiedOpcode::InitOp(offset, size) => format!("init {} {}", offset, size),
//         VerifiedOpcode::MallocOp(size) => format!("malloc {}", size),
//         VerifiedOpcode::ProjOp(offset, size) => format!("proj {} {}", offset, size),
//         VerifiedOpcode::CallOp => "call".to_owned(),
//         VerifiedOpcode::PrintOp => "print".to_owned(),
//         VerifiedOpcode::LitOp(lit) => format!("lit {}", lit),
//         VerifiedOpcode::GlobalFuncOp(label) => format!("global {}", label),
//         VerifiedOpcode::HaltOp(code) => format!("halt {}", code),
//         VerifiedOpcode::NewRgnOp => "new_rgn".to_owned(),
//         VerifiedOpcode::FreeRgnOp => "free_rgn".to_owned(),
//     }
// }

/// Get a pretty string representation of a compile-time region value.
pub fn region(r: &Region) -> String {
    match r {
        Region::HeapRgn => "Heap".to_string(),
        VarRgn(id) => "r".to_owned() + &id.1.to_string(),
    }
}

/// Get a pretty string representation of a kind context
/// (the kind and bound assignments of polymorphism variables).
pub fn kind_context(kinds: &KindContext) -> String {
    kinds
        .iter()
        .map(|entry| match entry {
            CapabilityKindContextEntry(id, bound) => {
                let prefix = "c".to_owned() + &id.0.to_string() + "_" + &id.1.to_string();
                if bound.len() != 0 {
                    return prefix + "≤" + &caps(bound);
                } else {
                    return prefix;
                }
            }
            RegionKindContextEntry(id) => "r".to_owned() + &id.1.to_string(),
            TypeKindContextEntry(id, repr) => "t".to_owned() + &id.0.to_string() + "_" + &id.1.to_string() + ": " + &representation(repr),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Get a pretty string representation of a capability set.
pub fn caps(cs: &Vec<Capability>) -> String {
    "{".to_owned()
        + &cs
            .iter()
            .map(|c| cap(c.clone()))
            .collect::<Vec<_>>()
            .join(",")
        + "}"
}

/// Get a pretty string representation of a capability.
pub fn cap(c: Capability) -> String {
    match c {
        VarCap(id) => "c".to_owned() + &id.1.to_string(),
        UniqueCap(r) => "1".to_owned() + &region(&r),
        ReadWriteCap(r) => "+".to_owned() + &region(&r),
    }
}

/// Get a pretty string representation of a list of types.
pub fn types(ts: &Vec<Type>) -> String {
    ts.iter().map(|t| typ(t)).collect::<Vec<_>>().join(", ")
}

/// Get a pretty string representation of a type.
pub fn typ(t: &Type) -> String {
    match t {
        I32Type => "i32".to_string(),
        HandleType(r) => "handle(".to_owned() + &region(r) + ")",
        MutableType(t) => "mut ".to_owned() + &typ(t),
        TupleType(ts, r) => "(".to_owned() + &types(ts) + ")@" + &region(r),
        ArrayType(t, r) => "[".to_owned() + &typ(t) + "]@" + &region(r),
        VarType(id, _repr) => "t".to_owned() + &id.1.to_string(),
        ExistsType(id, repr, t) => "Exists t".to_owned() + &id.1.to_string() + ": " + &representation(repr) + ". " + &typ(t),
        FuncType(kinds, c, ts) => {
            let quantification = if kinds.len() == 0 {
                "".to_owned()
            } else {
                "Forall ".to_owned() + &kind_context(kinds) + ". "
            };
            let body = "[".to_owned() + &caps(c) + "](" + &types(ts) + ")->0";
            quantification.to_owned() + &body
        }
        GuessType(l) => format!("Guess {}", l),
    }
}

/// Get a pretty string representation of the kind of the given compile-time value.
pub fn get_kind_str(ctval: &CTStackVal) -> String {
    kind(get_kind(ctval))
}

/// Get a pretty string representation of a byte, interpreting it as an instruction.
pub fn op_u8(byte: u8) -> String {
    (match byte {
        0x00 => "req",
        0x01 => "region",
        0x02 => "heap",
        0x03 => "cap",
        0x04 => "cap_le",
        0x05 => "unique",
        0x06 => "rw",
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
        0x1A => "print",
        0x1B => "int",
        0x1C => "global",
        0x1D => "halt",
        0x1E => "pack",
        0x1F => "word32",
        0x20 => "word64",
        0x21 => "ptr",
        0x22 => "reprs",
        0x23 => "new_rgn",
        0x24 => "free_rgn",
        0x25 => "forall",
        0x26 => "llarof",
        0x27 => "rgn_poly",
        0x28 => "ylop_ngr",
        _ => panic!("unknown opcode {}", byte),
    })
    .to_owned()
}

/// Get a pretty string representation of an unverified opcode.
pub fn unverified_op(op: UnverifiedOpcode) -> String {
    match op {
        ReqOp => "req".to_owned(),
        RegionOp => "region".to_owned(),
        HeapOp => "heap".to_owned(),
        CapOp => "cap".to_owned(),
        CapLEOp => "cap_le".to_owned(),
        UniqueOp => "unique".to_owned(),
        RWOp => "rw".to_owned(),
        BothOp => "both".to_owned(),
        HandleOp => "handle".to_owned(),
        I32Op => "i32".to_owned(),
        EndFunctionOp => "END_FUNC".to_owned(),
        MutOp => "mut".to_owned(),
        TupleOp(n) => format!("tuple {}", n.to_string()),
        ArrOp => "arr".to_owned(),
        AllOp => "all".to_owned(),
        SomeOp => "some".to_owned(),
        EmosOp => "emos".to_owned(),
        FuncOp(n) => format!("func {}", n.to_string()),
        CTGetOp(n) => format!("ct_get {}", n.to_string()),
        CTPopOp => "ct_pop".to_owned(),
        UnpackOp => "unpack".to_owned(),
        UnverifiedOpcode::GetOp(n) => format!("get {}", n.to_string()),
        UnverifiedOpcode::InitOp(n) => format!("init {}", n.to_string()),
        UnverifiedOpcode::MallocOp => "malloc".to_owned(),
        UnverifiedOpcode::ProjOp(n) => format!("proj {}", n.to_string()),
        UnverifiedOpcode::CallOp => "call".to_owned(),
        UnverifiedOpcode::PrintOp => "print".to_owned(),
        UnverifiedOpcode::LitOp(lit) => format!("lit {}", lit.to_string()),
        UnverifiedOpcode::GlobalFuncOp(label) => format!("global {}", label),
        UnverifiedOpcode::HaltOp(code) => format!("halt {}", code),
        PackOp => "pack".to_owned(),
        Word32Op => "word32".to_owned(),
        Word64Op => "word64".to_owned(),
        PtrOp => "ptr".to_owned(),
        ReprsOp(n) => format!("reprs {}", n.to_string()),
        NewRgnOp => "new_rgn".to_owned(),
        FreeRgnOp => "free_rgn".to_owned(),
        ForallOp => "forall".to_owned(),
        LlarofOp => "llarof".to_owned(),
        RgnPolyOp => "rgn_poly".to_owned(),
        YlopNgrOp => "ngr_poly".to_owned(),
    }
}

/// Get a pretty string representation of a kind.
pub fn kind(k: Kind) -> String {
    (match k {
        Kind::CapabilityKind => "capability",
        Kind::RegionKind => "region",
        Kind::TypeKind => "type",
        Kind::ReprKind => "repr",
    })
    .to_owned()
}

pub fn representation(repr: &Repr) -> String {
    match repr {
        Repr::Word32Repr => "32bit".to_owned(),
        Repr::Word64Repr => "64bit".to_owned(),
        Repr::PtrRepr => "ptr".to_owned(),
        Repr::TupleRepr(reprs) => "Tuple(".to_owned() + &reprs.iter().map(|r| representation(r)).collect::<Vec<_>>().join(", ") + ")"
    }
}

pub fn ctval(ctv: &CTStackVal) -> String {
    match ctv {
        CTStackVal::TypeCTStackVal(t) => typ(t),
        CTStackVal::RegionCTStackVal(r) => region(r),
        CTStackVal::CapCTStackVal(cs) => caps(cs),
        CTStackVal::ReprCTStackVal(r) => representation(r),
    }
}