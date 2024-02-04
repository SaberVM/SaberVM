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
use crate::header::Type;
use crate::header::Type::*;
use crate::header::UnverifiedOpcode;
use crate::header::UnverifiedOpcode::*;
use crate::header::VerifiedOpcode;

/// Get a pretty string representation of a verified opcode.
pub fn verified_op(op: &VerifiedOpcode) -> String {
    match op {
        VerifiedOpcode::GetOp(offset, size) => "get ".to_owned() + &offset.to_string() + " " + &size.to_string(),
        VerifiedOpcode::InitOp(offset) => "init ".to_owned() + &offset.to_string(),
        VerifiedOpcode::MallocOp(size) => "malloc ".to_owned() + &size.to_string(),
        VerifiedOpcode::ProjOp(offset) => "proj ".to_owned() + &offset.to_string(),
        VerifiedOpcode::CallOp => "call".to_owned(),
        VerifiedOpcode::PrintOp => "print".to_owned(),
        VerifiedOpcode::LitOp(lit) => "lit ".to_owned() + &lit.to_string(),
        VerifiedOpcode::GlobalFuncOp(label) => "global ".to_owned() + &label.to_string(),
        VerifiedOpcode::HaltOp(code) => "halt ".to_owned() + &code.to_string(),
    }
}

/// Get a pretty string representation of a compile-time region value.
pub fn region(r: Region) -> String {
    match r {
        Region::HeapRgn => "Heap".to_string(),
        VarRgn(id) => "r".to_owned() + &id.0.to_string() + "_" + &id.1.to_string(),
    }
}

/// Get a pretty string representation of a kind context
/// (the kind and bound assignments of polymorphism variables).
pub fn kind_context(kinds: &KindContext) -> String {
    kinds
        .iter()
        .map(|entry| match entry {
            CapabilityKindContextEntry(id, bound) => {
                let prefix = "c".to_owned() + &id.1.to_string();
                if bound.len() != 0 {
                    return prefix + "≤" + &caps(bound);
                } else {
                    return prefix;
                }
            }
            RegionKindContextEntry(id) => "r".to_owned() + &id.1.to_string(),
            TypeKindContextEntry(id) => "t".to_owned() + &id.1.to_string(),
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
        UniqueCap(r) => "1".to_owned() + &region(r),
        ReadWriteCap(r) => "+".to_owned() + &region(r),
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
        HandleType(r) => "handle(".to_owned() + &region(*r) + ")",
        MutableType(t) => "mut ".to_owned() + &typ(t),
        TupleType(ts, r) => "(".to_owned() + &types(ts) + ")@" + &region(*r),
        ArrayType(t) => "[]".to_owned() + &typ(t),
        VarType(id) => "t".to_owned() + &id.1.to_string(),
        ExistsType(id, t) => "Exists t".to_owned() + &id.1.to_string() + ". " + &typ(t),
        FuncType(kinds, c, ts) => {
            let quantification = if kinds.len() == 0 {
                "".to_owned()
            } else {
                "Forall ".to_owned() + &kind_context(kinds) + ". "
            };
            let body = "[".to_owned() + &caps(c) + "](" + &types(ts) + ")->0";
            quantification.to_owned() + &body
        }
        GuessType(_) => panic!("type-checking artifact lasted too long"),
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
    }
}

/// Get a pretty string representation of a kind.
pub fn kind(k: Kind) -> String {
    (match k {
        Kind::CapabilityKind => "capability",
        Kind::RegionKind => "region",
        Kind::TypeKind => "type",
    })
    .to_owned()
}
