/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod header;
mod parse;
mod verify;
mod vm;

use std::fs;

fn go(bytes: header::ByteStream) -> Result<(), header::Error> {
    let stmt1s = parse::go(&bytes)?;
    let stmt2s = verify::go(stmt1s)?;
    vm::go(stmt2s);
    Ok(())
}
 
fn main() {
    // get the bytes from the local bin.svm file (later this will be a CLI arg of course)
    let bytes: header::ByteStream = fs::read("bin.svm").unwrap();
    let res = go(bytes);
    let _ = dbg!(res);
}
 