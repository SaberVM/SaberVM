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
    fn vm_function(bytes: *mut u8) -> u8;
}

pub fn go(stmts: Vec<VerifiedStmt>) {
    dbg!(unsafe {vm_function(merge_stmts(stmts).as_mut_ptr())});
}

fn merge_stmts(stmts: Vec<VerifiedStmt>) -> Vec<u8> {
    let mut merged: Vec<u8> = Vec::new();
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

fn op_to_bytes(op: &VerifiedOpcode) -> Vec<u8> {
    match op {
        GetOp(offset) => vec![0x00, *offset],
        InitOp(offset) => vec![0x01, *offset],
        MallocOp(size) => vec![0x02, *size],
        ProjOp(offset) => vec![0x03, *offset],
        CallOp => vec![0x04],
        PrintOp => vec![0x05],
        LitOp(lit) => {
            let l = *lit as u32;
            vec![0x06, (l >> 24) as u8, (l >> 16) as u8, (l >> 8) as u8, l as u8]
        }
        GlobalFuncOp(label) => {
            let l = *label as u32;
            vec![0x07, (l >> 24) as u8, (l >> 16) as u8, (l >> 8) as u8, l as u8]
        }
        HaltOp(code) => vec![0x08, *code],
    }
}