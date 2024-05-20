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

// const BYTES: [u8; 23] = [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x09, 0x00, 0x0B, 0x13, 0x07, 0x00, 0x00, 0x00, 0x12, 0x13, 0x00, 0x00, 0x00, 0x00, 0x15];

fn go(bytes: header::ByteStream) -> Result<(), header::Error> {
    let (data_section_len, data_section, types_instrs, unverified_stmts) = parse::go(&bytes)?;
    let stmts = verify::go(data_section_len, types_instrs, unverified_stmts)?;
    vm::go(data_section_len, data_section, stmts);
    Ok(())
}

fn main() {
    // get the bytes from the local bin.svm file (later this will be a CLI arg of course)
    let bytes: header::ByteStream = fs::read("bin.svm").unwrap();
    let res = go(bytes);
    match res {
        Ok(_) => println!("Success!"),
        Err(e) => println!("{}", error_msgs::msg(e)),
    }
}
