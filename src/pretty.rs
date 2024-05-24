/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::*;

pub trait Pretty {
    fn pretty(&self) -> String;
}

impl Pretty for Op1 {
    fn pretty(&self) -> String {
        match self {
            Op1::Unique => "unique".to_string(),
            Op1::Handle => "handle".to_string(),
            Op1::I32 => "i32".to_string(),
            Op1::Tuple(n) => "tuple ".to_string() + &n.to_string(),
            Op1::Some => "some".to_string(),
            Op1::All => "all".to_string(),
            Op1::Rgn => "rgn".to_string(),
            Op1::End => "end".to_string(),
            Op1::App => "app".to_string(),
            Op1::Func(n) => "func ".to_string() + &n.to_string(),
            Op1::CTGet(n) => "ctget ".to_string() + &n.to_string(),
            Op1::Lced => "lced".to_string(),
            Op1::Unpack => "unpack".to_string(),
            Op1::Get(n) => "get ".to_string() + &n.to_string(),
            Op1::Init(n) => "init ".to_string() + &n.to_string(),
            Op1::Malloc => "malloc".to_string(),
            Op1::Proj(n) => "proj ".to_string() + &n.to_string(),
            Op1::Call => "call".to_string(),
            Op1::Print => "print".to_string(),
            Op1::Lit(i) => i.to_string(),
            Op1::GlobalFunc(n) => "global_func ".to_string() + &n.to_string(),
            Op1::Halt => "halt".to_string(),
            Op1::Pack => "pack".to_string(),
            Op1::Size(n) => "size ".to_string() + &n.to_string(),
            Op1::NewRgn(n) => "new_rgn ".to_string() + &n.to_string(),
            Op1::FreeRgn => "free_rgn".to_string(),
            Op1::Ptr => "ptr".to_string(),
            Op1::Deref => "deref".to_string(),
            Op1::Arr => "arr".to_string(),
            Op1::ArrMut => "arr_mut".to_string(),
            Op1::ArrProj => "arr_proj".to_string(),
            Op1::Add => "add".to_string(),
            Op1::Mul => "mul".to_string(),
            Op1::Div => "div".to_string(),
            Op1::CallNZ => "call_nz".to_string(),
            Op1::Data(n) => "data ".to_string() + &n.to_string(),
            Op1::DataSec => "data_sec".to_string(),
            Op1::U8 => "u8".to_string(),
            Op1::CopyN => "copy_n".to_string(),
            Op1::U8Lit(n) => "u8_lit ".to_string() + &n.to_string(),
            Op1::U8ToI32 => "u8_to_i32".to_string(),
            Op1::Import(a, b) => "import ".to_string() + &int_pair_to_str(a, b),
            Op1::Export(a, b) => "export ".to_string() + &int_pair_to_str(a, b),
            Op1::Modulo => "modulo".to_string(),
        }
    }
}

fn int_pair_to_str(a: &u64, b: &u64) -> String {
    return std::str::from_utf8(&a.to_le_bytes()).unwrap().to_owned() + &std::str::from_utf8(&b.to_le_bytes()).unwrap();
}

impl Pretty for Op2 {
    fn pretty(&self) -> String {
        match self {
            Op2::Get(s1, s2) => "get ".to_string() + &s1.to_string() + " " + &s2.to_string(),
            Op2::Init(s1, s2, s3) => "init ".to_string() + &s1.to_string() + " " + &s2.to_string() + " " + &s3.to_string(),
            Op2::InitIP(s1, s2) => "init_ip ".to_string() + &s1.to_string() + " " + &s2.to_string(),
            Op2::Malloc(s) => "malloc ".to_string() + &s.to_string(),
            Op2::Alloca(s) => "alloca ".to_string() + &s.to_string(),
            Op2::Proj(s1, s2, s3) => "proj ".to_string() + &s1.to_string() + " " + &s2.to_string() + " " + &s3.to_string(),
            Op2::ProjIP(s1, s2) => "proj_ip ".to_string() + &s1.to_string() + " " + &s2.to_string(),
            Op2::Call => "call".to_string(),
            Op2::Print => "print".to_string(),
            Op2::Lit(i) => "lit ".to_string() + &i.to_string(),
            Op2::GlobalFunc(n) => "global_func ".to_string() + &n.to_string(),
            Op2::Halt => "halt".to_string(),
            Op2::NewRgn(n) => "new_rgn ".to_string() + &n.to_string(),
            Op2::FreeRgn => "free_rgn".to_string(),
            Op2::Deref(s) => "deref ".to_string() + &s.to_string(),
            Op2::NewArr(s) => "new_arr ".to_string() + &s.to_string(),
            Op2::ArrMut(s) => "arr_mut ".to_string() + &s.to_string(),
            Op2::ArrProj(s) => "arr_proj ".to_string() + &s.to_string(),
            Op2::AddI32 => "add_i32".to_string(),
            Op2::MulI32 => "mul_i32".to_string(),
            Op2::DivI32 => "div_i32".to_string(),
            Op2::CallNZ => "call_nz".to_string(),
            Op2::Data(s) => "data ".to_string() + &s.to_string(),
            Op2::DataIndex(s) => "data_index ".to_string() + &s.to_string(),
            Op2::CopyN(s) => "copy_n ".to_string() + &s.to_string(),
            Op2::U8Lit(n) => "u8_lit ".to_string() + &n.to_string(),
            Op2::AddU8 => "add_u8".to_string(),
            Op2::MulU8 => "mul_u8".to_string(),
            Op2::DivU8 => "div_u8".to_string(),
            Op2::U8ToI32 => "u8_to_i32".to_string(),
            Op2::ModuloI32 => "modulo_i32".to_string(),
            Op2::ModuloU8 => "modulo_u8".to_string(),
        }
    }
}

impl Pretty for Stmt1 {
    fn pretty(&self) -> String {
        match self {
            Stmt1::Func(id, _pos, ops) => "fn foo".to_string() + &id.to_string() + " = " + &ops.iter().map(Op1::pretty).collect::<Vec<String>>().join("; "),
        }
    }
}

impl Pretty for Stmt2 {
    fn pretty(&self) -> String {
        match self {
            Stmt2::Func(pos, t, ops) => "fn foo".to_string() + &pos.to_string() + ": " + &t.pretty() + " = " + &ops.iter().map(|op|op.pretty()).collect::<Vec<String>>().join("; "),
        }
    }
}

impl Pretty for RgnId {
    fn pretty(&self) -> String {
        match self {
            RgnId::Var(id) => "r".to_string() + &id.1.to_string(),
            RgnId::DataSection => "data_section".to_string(),
        }
    }
}

impl Pretty for Region {
    fn pretty(&self) -> String {
        match self {
            Region { id, .. } => id.pretty()
        }
    }
}

impl Pretty for Type {
    fn pretty(&self) -> String {
        match self {
            Type::I32 => "i32".to_string(),
            Type::U8 => "u8".to_string(),
            Type::Handle(r) => "handle(".to_string() + &r.pretty() + ")",
            Type::Tuple(ts) => "(".to_string() + &ts.iter().map(|(_, t)| t.pretty()).collect::<Vec<String>>().join(", ") + ")",
            Type::Ptr(t, r) => t.pretty() + "@" + &r.pretty(),
            Type::Var(id, _) => "a".to_string() + &id.1.to_string(),
            Type::Func(ts) => "(".to_string() + &ts.iter().map(|t| t.pretty()).collect::<Vec<String>>().join(", ") + ")->0",
            Type::Forall(id, size, t) => "forall a".to_string() + &id.1.to_string() + ": " + &size.to_string() + "byte. " + &t.pretty(),
            Type::ForallRegion(r, t, _) => "forall ".to_string() + &r.pretty() + ": Rgn" + own_suffix(r) + ". " + &t.pretty(),
            Type::Exists(id, size, t) => "exists a".to_string() + &id.1.to_string() + ": " + &size.to_string() + "byte. " + &t.pretty(),
            Type::Array(t, r) => t.pretty() + "[]@" + &r.pretty(),
        }
    }
}

fn own_suffix(r: &Region) -> &str {
    match r {
        Region { unique, .. } => if *unique { "!" } else { "" },
    }
}

impl Pretty for Kind {
    fn pretty(&self) -> String {
        match self {
            Kind::Type => "Type".to_string(),
            Kind::Region => "Region".to_string(),
            Kind::Size => "Size".to_string(),
        }
    }
}

