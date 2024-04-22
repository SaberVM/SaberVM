/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::vec;

use crate::header::*;

extern "C" {
    fn vm_function(bytes: *mut u8, len: usize) -> u8;
}

pub fn go(stmts: Vec<Stmt2>) {
    let mut code = merge_stmts(stmts);
    let len = code.len();
    dbg!(unsafe { vm_function(code.as_mut_ptr(), len) });
}

fn merge_stmts(stmts: Vec<Stmt2>) -> Vec<u8> {
    let mut merged: Vec<u8> = Vec::new();
    let mut label_map = HashMap::new();
    let mut next_pos: u32 = 0;
    stmts.iter().for_each(|stmt| {
        let Stmt2::Func(label, _, opcodes) = stmt;
        label_map.insert(label, next_pos);
        let mut size = 0;
        for op in opcodes {
            size += op_to_bytes(op).len();
        }
        next_pos += size as u32;
    });
    for Stmt2::Func(_, _, opcodes) in &stmts {
        for op in opcodes {
            match op {
                Op2::GlobalFunc(label) => merged.extend(op_to_bytes(&Op2::GlobalFunc(
                    *label_map.get(label).unwrap(),
                ))),
                _ => merged.extend(op_to_bytes(op)),
            }
        }
    }
    merged
}

fn op_to_bytes(op: &Op2) -> Vec<u8> {
    match op {
        Op2::Get(offset, size) => [
            vec![0],
            offset.to_le_bytes().to_vec(),
            size.to_le_bytes().to_vec(),
        ]
        .concat(),
        Op2::Init(offset, size, tpl_size) => [
            vec![1],
            offset.to_le_bytes().to_vec(),
            size.to_le_bytes().to_vec(),
            tpl_size.to_le_bytes().to_vec(),
        ]
        .concat(),
        Op2::InitIP(offset, size) => [
            vec![2],
            offset.to_le_bytes().to_vec(),
            size.to_le_bytes().to_vec(),
        ]
        .concat(),
        Op2::Malloc(size) => [vec![3], size.to_le_bytes().to_vec()].concat(),
        Op2::Alloca(size) => [vec![4], size.to_le_bytes().to_vec()].concat(),
        Op2::Proj(offset, size, tpl_size) => [
            vec![5],
            offset.to_le_bytes().to_vec(),
            size.to_le_bytes().to_vec(),
            tpl_size.to_le_bytes().to_vec(),
        ]
        .concat(),
        Op2::ProjIP(offset, size) => [
            vec![6],
            offset.to_le_bytes().to_vec(),
            size.to_le_bytes().to_vec(),
        ]
        .concat(),
        Op2::Call => vec![7],
        Op2::Print => vec![8],
        Op2::Lit(lit) => [vec![9], lit.to_le_bytes().to_vec()].concat(),
        Op2::GlobalFunc(label) => [vec![10], label.to_le_bytes().to_vec()].concat(),
        Op2::Halt => vec![11],
        Op2::NewRgn => vec![12],
        Op2::FreeRgn => vec![13],
        Op2::Deref(size) => [vec![14], size.to_le_bytes().to_vec()].concat(),
    }
}
