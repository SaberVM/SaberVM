/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::vec;

use crate::header::*;

extern "C" {
    fn vm_function(bytes: *mut u32, len: usize) -> u8;
}

pub fn go(stmts: Vec<Stmt2>) {
    let mut code = merge_stmts(stmts);
    let len = code.len();
    dbg!(unsafe { vm_function(code.as_mut_ptr(), len) });
}

fn merge_stmts(stmts: Vec<Stmt2>) -> Vec<u32> {
    let mut merged: Vec<u32> = Vec::new();
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
                Op2::GlobalFunc(label) => {
                    merged.extend(op_to_bytes(&Op2::GlobalFunc(*label_map.get(label).unwrap())))
                }
                _ => merged.extend(op_to_bytes(op)),
            }
        }
    }
    merged
}

fn op_to_bytes(op: &Op2) -> Vec<u32> {
    match op {
        Op2::Get(offset, size) => vec![0, *offset as u32, *size as u32],
        Op2::Init(offset, size) => vec![1, *offset as u32, *size as u32],
        Op2::Malloc(size) => vec![2, *size as u32],
        Op2::Proj(offset, size) => vec![3, *offset as u32, *size as u32],
        Op2::Call => vec![4],
        Op2::Print => vec![5],
        Op2::Lit(lit) => vec![6, *lit as u32],
        Op2::GlobalFunc(label) => vec![7, *label as u32],
        Op2::Halt => vec![8],
        Op2::NewRgn => vec![9],
        Op2::FreeRgn => vec![10],
    }
}
