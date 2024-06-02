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
mod aot;

use std::fs;
use std::env;

fn go(bytes: Vec<header::ByteStream>) -> Result<(), header::Error> {
    let mut ir_programs = vec![];
    for prog in bytes {
        let (data_section, types_instrs, unverified_stmts) = parse::go(&prog)?;
        let ir_program = verify::go(data_section, types_instrs, unverified_stmts)?;
        ir_programs.push(ir_program);
    }
    vm::go(ir_programs);
    Ok(())
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let bytes: Vec<header::ByteStream> = args.iter().skip(1).map(|filename| fs::read(filename).unwrap()).collect();
    let res = go(bytes);
    match res {
        Ok(_) => println!("Success!"),
        Err(e) => println!("{}", error_msgs::msg(e)),
    }
}
