/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod error_handling;
mod header;
mod parse;
mod pretty;
mod verify;

use std::fs;

use crate::header::ByteStream;
use crate::header::Error;
use crate::header::VerifiedStmt::*;

fn go(bytes: &ByteStream) -> Result<(), Error> {
    let unverified_stmts = parse::go(bytes)?;

    let verified_stmts = verify::go(unverified_stmts)?;

    // the following is just for debugging
    for func in verified_stmts {
        let Func(label, func_type, ops) = func;
        dbg!(label);
        dbg!(pretty::typ(&func_type));
        for op in ops {
            println!("{}", pretty::verified_op(&op))
        }
    }

    Ok(())
}

fn main() {
    // get the bytes from the local bin.svm file (later this will be a CLI arg of course)
    let bytes = fs::read("bin.svm").unwrap();

    let mb_error = go(&bytes);

    error_handling::handle(mb_error);
}
