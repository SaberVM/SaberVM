/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::vec;

use crate::header::{VerifiedOpcode, VerifiedStmt};
use crate::header::VerifiedOpcode::*;
use crate::header::VerifiedStmt::*;

extern "C" {
    fn vm_function(bytes: *mut u32, len: usize) -> u8;
}

pub fn go(stmts: Vec<VerifiedStmt>) {
    let mut code = merge_stmts(stmts);
    let len = code.len();
    dbg!(unsafe {vm_function(code.as_mut_ptr(), len)});
}

fn merge_stmts(stmts: Vec<VerifiedStmt>) -> Vec<u32> {
    let mut merged: Vec<u32> = Vec::new();
    let mut label_map = HashMap::new();
    let mut next_pos: u32 = 0;
    stmts.iter().for_each(|stmt| {
        let Func(label, _, opcodes) = stmt;
        label_map.insert(label, next_pos);
        let mut size = 0;
        for op in opcodes {
            size += op_to_bytes(op).len();
        }
        next_pos += size as u32;
    });
    for Func(_, _, opcodes) in &stmts {
        for op in opcodes {
            match op {
                GlobalFuncOp(label) => merged.extend(op_to_bytes(&GlobalFuncOp(*label_map.get(label).unwrap()))),
                _ => merged.extend(op_to_bytes(op)),
            }
        }
    }
    merged
}

fn op_to_bytes(op: &VerifiedOpcode) -> Vec<u32> {
    match op {
        GetOp(offset, size) => vec![0, *offset as u32, *size as u32],
        InitOp(offset, size) => vec![1, *offset as u32, *size as u32],
        MallocOp(size) => vec![2, *size as u32],
        ProjOp(offset, size) => vec![3, *offset as u32, *size as u32],
        CallOp => vec![4],
        PrintOp => vec![5],
        LitOp(lit) => vec![6, *lit as u32],
        GlobalFuncOp(label) => vec![7, *label as u32],
        HaltOp(code) => vec![8, *code as u32],
        NewRgnOp => vec![9],
        FreeRgnOp => vec![10],
    }
}