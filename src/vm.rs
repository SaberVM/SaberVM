/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::vec;

use crate::header::*;
use crate::pretty::Pretty;
use std::fs;

extern "C" {
    fn vm_function(bytes: *mut u8) -> u8;
}

pub fn go(ir_programs: Vec<IRProgram>) -> u8 {
    let mut str = String::new();
    let code_size = 4 + ir_programs.iter().map(program_size).sum::<usize>();
    let mut code = Vec::with_capacity(code_size);
    let mut import_map = HashMap::new();
    let mut prog_id = 0;
    for prog in &ir_programs {
        for (k,v) in &prog.exports {
            import_map.insert(*k, (prog_id, *v));
        }
        prog_id += 1;
    }
    let mut data_sec_positions = HashMap::new();
    code.extend(vec![0, 0, 0, 0]);
    let mut pos: u32 = 4;
    prog_id = 0;
    for prog in &ir_programs {
        data_sec_positions.insert(prog_id, pos - 4);
        let data_section_len = prog.data_section.len();
        code.extend(prog.data_section.iter());
        pos += data_section_len as u32;
        prog_id += 1;
    }
    code[0..4].copy_from_slice(&(pos - 4).to_le_bytes());
    let mut func_positions = HashMap::new();
    let mut pos2 = pos;
    prog_id = 0;
    for prog in &ir_programs {
        for Stmt2::Func(l, _, ops) in &prog.funcs {
            func_positions.insert((prog_id, *l), pos2);
            pos2 += ops.iter().map(op_len).sum::<usize>() as u32;
        }
        prog_id += 1;
    }
    assert!(pos2 == code_size as u32);
    assert!(pos < pos2);
    prog_id = 0;
    for prog in &ir_programs {
        let mut label_map = HashMap::new();
        let mut pos2 = pos;
        for Stmt2::Func(label, _, ops) in &prog.funcs {
            label_map.insert(*label, pos2);
            pos2 += ops.iter().map(op_len).sum::<usize>() as u32;
        }
        for Stmt2::Func(l, t, ops) in &prog.funcs {
            str += &("function ".to_string() + &l.to_string() + ": " + &t.pretty() + "\n");
            for op in ops {
                str += &(pos.to_string() + " " + &op.pretty() + "\n");
                match op {
                    Op2::GlobalFunc(label) => {
                        let func_pos = match label_map.get(label) {
                            Some(pos) => *pos,
                            None => {
                                let func_id = import_map.get(prog.imports.get(label).unwrap()).unwrap();
                                *func_positions.get(func_id).unwrap()
                            },
                        };
                        code.extend(op_to_bytes(&Op2::GlobalFunc(func_pos as u32)));
                    }
                    Op2::Data(data_pos) => {
                        let data_sec_pos = data_sec_positions.get(&prog_id).unwrap();
                        code.extend(op_to_bytes(&Op2::Data(*data_sec_pos as usize + *data_pos)));
                    }
                    _ => code.extend(op_to_bytes(op)),
                }
                pos += op_len(op) as u32;
            }
        }
        prog_id += 1;
    }
    let _ = fs::write("t.txt", str);
    unsafe { vm_function(code.as_mut_ptr()) }
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
        Op2::NewRgn(size) => [vec![12], size.to_le_bytes().to_vec()].concat(),
        Op2::FreeRgn => vec![13],
        Op2::Deref(size) => [vec![14], size.to_le_bytes().to_vec()].concat(),
        Op2::NewArr(size) => [vec![15], size.to_le_bytes().to_vec()].concat(),
        Op2::ArrMut(size) => [vec![16], size.to_le_bytes().to_vec()].concat(),
        Op2::ArrProj(size) => [vec![17], size.to_le_bytes().to_vec()].concat(),
        Op2::AddI32 => vec![18],
        Op2::MulI32 => vec![19],
        Op2::DivI32 => vec![20],
        Op2::CallNZ => vec![21],
        Op2::Data(size) => [vec![22], size.to_le_bytes().to_vec()].concat(),
        Op2::DataIndex(size) => [vec![23], size.to_le_bytes().to_vec()].concat(),
        Op2::CopyN(size) => [vec![24], size.to_le_bytes().to_vec()].concat(),
        Op2::U8Lit(n) => vec![25, *n],
        Op2::AddU8 => vec![26],
        Op2::MulU8 => vec![27],
        Op2::DivU8 => vec![28],
        Op2::U8ToI32 => vec![29],
        Op2::ModuloI32 => vec![30],
        Op2::ModuloU8 => vec![31],
        Op2::I32ToU8 => vec![32],
        Op2::Read(c) => vec![33, *c],
        Op2::Write(c) => vec![34, *c],
    }
}


fn op_len(op: &Op2) -> usize {
    match op {
        Op2::Get(_, _) => 1 + 8 + 8,
        Op2::Init(_, _, _) => 1 + 8 + 8 + 8,
        Op2::InitIP(_, _) => 1 + 8 + 8,
        Op2::Malloc(_) => 1 + 8,
        Op2::Alloca(_) => 1 + 8,
        Op2::Proj(_, _, _) => 1 + 8 + 8 + 8,
        Op2::ProjIP(_, _) => 1 + 8 + 8,
        Op2::Call => 1,
        Op2::Print => 1,
        Op2::Lit(_) => 1 + 4,
        Op2::GlobalFunc(_) => 1 + 4,
        Op2::Halt => 1,
        Op2::NewRgn(_) => 1 + 8,
        Op2::FreeRgn => 1,
        Op2::Deref(_) => 1 + 8,
        Op2::NewArr(_) => 1 + 8,
        Op2::ArrMut(_) => 1 + 8,
        Op2::ArrProj(_) => 1 + 8,
        Op2::AddI32 => 1,
        Op2::MulI32 => 1,
        Op2::DivI32 => 1,
        Op2::CallNZ => 1,
        Op2::Data(_) => 1 + 8,
        Op2::DataIndex(_) => 1 + 8,
        Op2::CopyN(_) => 1 + 8,
        Op2::U8Lit(_) => 1 + 1,
        Op2::AddU8 => 1,
        Op2::MulU8 => 1,
        Op2::DivU8 => 1,
        Op2::U8ToI32 => 1,
        Op2::ModuloI32 => 1,
        Op2::ModuloU8 => 1,
        Op2::I32ToU8 => 1,
        Op2::Read(_) => 1 + 1,
        Op2::Write(_) => 1 + 1,
    }
}

fn program_size(prog: &IRProgram) -> usize {
    let mut out = prog.data_section.len();
    for Stmt2::Func(_, _, ops) in &prog.funcs {
        out += ops.iter().map(op_len).sum::<usize>();
    }
    out
}