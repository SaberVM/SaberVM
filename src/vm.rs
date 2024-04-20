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
        Op2::Get(offset, size) => vec![
            0,
            (*offset >> 24) as u8,
            (*offset >> 16) as u8,
            (*offset >> 8) as u8,
            *offset as u8,
            (*size >> 24) as u8,
            (*size >> 16) as u8,
            (*size >> 8) as u8,
            *size as u8,
        ],
        Op2::Init(offset, size) => vec![
            1,
            (*offset >> 24) as u8,
            (*offset >> 16) as u8,
            (*offset >> 8) as u8,
            *offset as u8,
            (*size >> 24) as u8,
            (*size >> 16) as u8,
            (*size >> 8) as u8,
            *size as u8,
        ],
        Op2::Malloc(size) => vec![
            2,
            (*size >> 24) as u8,
            (*size >> 16) as u8,
            (*size >> 8) as u8,
            *size as u8,
        ],
        Op2::Proj(offset, size, tpl_size) => vec![
            3,
            (*offset >> 24) as u8,
            (*offset >> 16) as u8,
            (*offset >> 8) as u8,
            *offset as u8,
            (*size >> 24) as u8,
            (*size >> 16) as u8,
            (*size >> 8) as u8,
            *size as u8,
            (*tpl_size >> 24) as u8,
            (*tpl_size >> 16) as u8,
            (*tpl_size >> 8) as u8,
            *tpl_size as u8,
        ],
        Op2::Call => vec![4],
        Op2::Print => vec![5],
        Op2::Lit(lit) => [vec![6], lit.to_le_bytes().to_vec()].concat(),
        Op2::GlobalFunc(label) => vec![
            7,
            (*label >> 24) as u8,
            (*label >> 16) as u8,
            (*label >> 8) as u8,
            *label as u8,
        ],
        Op2::Halt => vec![8],
        Op2::NewRgn => vec![9],
        Op2::FreeRgn => vec![10],
        Op2::Deref(size) => vec![
            11,
            (*size >> 24) as u8,
            (*size >> 16) as u8,
            (*size >> 8) as u8,
            *size as u8,
        ],
    }
}
