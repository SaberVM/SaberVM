/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod header;
mod pretty;
mod error_msgs;
mod parse;
mod verify;
mod vm;

use std::fs;
use std::env;
use std::process::exit;

fn go(bytes: Vec<header::ByteStream>) -> Result<(), header::Error> {
    let mut ir_programs = vec![];
    for prog in bytes {
        let (data_section, types_instrs, unverified_stmts) = parse::go(&prog)?;
        // println!("{}", unverified_stmts.iter().map(|f|f.pretty() + "\n").collect::<String>());
        let ir_program = verify::go(data_section, types_instrs, unverified_stmts)?;
        ir_programs.push(ir_program);
    }
    let status = vm::go(ir_programs);
    if status != 0 {
        exit(status.into());
    }
    Ok(())
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let bytes: Vec<header::ByteStream> = args.iter().skip(1).map(|filename| fs::read(filename).unwrap()).collect();
    let res = go(bytes);
    if let Err(e) = res {
        println!("{}", error_msgs::msg(e));
    }
}
