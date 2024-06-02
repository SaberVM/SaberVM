/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub fn go(bytecode: &Vec<u8>) -> String {
    let mut code_section = String::new();
    let mut bytes_iter = bytecode.iter();
    loop {
        match bytes_iter.next() {
            None => break,
            Some(b) => match b {
                0x00 => {
                    // get op
                    let offset = instr_param_usize(&mut bytes_iter);
                    let size = instr_param_usize(&mut bytes_iter);
                    code_section.push_str(format!("    mov rcx {}\n    mov rdx {}\n    mov rsi, [rbp - rcx - rdx]\n    mov rdi, rsp\n    mov rcx, rdx\n    rep movsb\n    sub rsp, rdx\n", offset, size).as_str());
                }
                0x01 => {
                    // init op
                    let offset = instr_param_usize(&mut bytes_iter);
                    let size = instr_param_usize(&mut bytes_iter);
                    let tpl_size = instr_param_usize(&mut bytes_iter);
                    code_section.push_str(format!("    mov rdi, {}\n    mov rsi, {}\n    mov rdx, {}\n    sub rsp, rsi\n    mov r8, rsp\n    mov r9, rsp\n    sub r9, rdx\n    add r9, rdi\n    mov rcx, rsi\n    rep movsb\n", offset, size, tpl_size).as_str());
                }
                _ => code_section.push_str("\n"),
            },
        }
    }
    code_section
}

fn instr_param_usize(bytes_iter: &mut std::slice::Iter<u8>) -> usize {
    let mut out: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for i in 0..8 {
        out[i] = *bytes_iter.next().unwrap();
    }
    usize::from_le_bytes(out)
}
