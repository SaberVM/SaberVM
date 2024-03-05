/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::Error;
use crate::header::Error::*;
use crate::pretty;

/// Print a helpful error message for the given error, if there is one.
pub fn handle(res: Result<(), Error>) {
    match res {
        Err(e) => match e {
            SyntaxErrorParamNeeded(pos, op) => {
                println!(
                    "Syntax error! The file ended just as I was expecting the parameter for `{}`.\n[{}]",
                    pretty::op_u8(op),
                    pos
                )
            }
            SyntaxErrorUnknownOp(pos, op) => {
                println!(
                    "Syntax error! I don't know of an opcode with the code `{}`.\n[{}]",
                    op, pos
                )
            }
            TypeErrorEmptyCTStack(pos, op) => {
                println!(
                    "Type error! `{}` needs a value from the compile-time stack, but at this point the compile-time stack has nothing in it.\n[{}]", 
                    pretty::unverified_op(op),
                    pos
                )
            }
            KindErrorReq(pos, val) => {
                println!(
                    "Kind error! `req` needs a type or region, but it is receiving a `{}`.\n[{}]",
                    pretty::get_kind_str(&val),
                    pos
                )
            }
            KindError(pos, op, expected, found) => {
                println!(
                    "Kind error! `{}` needs a `{}`, but it is receiving a `{}`.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::kind(expected),
                    pretty::get_kind_str(&found),
                    pos
                )
            }
            TypeErrorEmptyExistStack(pos, op) => {
                println!(
                    "Type error! `{}` is trying to use the existential stack, but at this point the existential stack has nothing in it.\n[{}]", 
                    pretty::unverified_op(op),
                    pos
                )
            }
            TypeErrorParamOutOfRange(pos, op) => {
                println!(
                    "Type error! `{}` is trying to index something, but the index is higher than the number of things in that something.\n[{}]", 
                    pretty::unverified_op(op),
                    pos
                )
            }
            TypeErrorExistentialExpected(pos, found) => {
                println!(
                    "Type error! I'm trying to unpack `{}`, which is not an existential type (and therefore can't be unpacked).\n[{}]",
                    pretty::typ(&found),
                    pos
                )
            }
            TypeErrorEmptyStack(pos, op) => {
                println!(
                    "Type error! `{}` needs something from the stack, but the stack at this point will be empty.\n[{}]",
                    pretty::unverified_op(op),
                    pos
                )
            }
            CapabilityError(pos, op, needed, present) => {
                println!(
                    "Capability error! `{}` doesn't have enough capabilities! It needs `{}` but has `{}`.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::caps(&needed),
                    pretty::caps(&present),
                    pos
                )
            }
            TypeErrorInit(pos, expected, found) => {
                println!(
                    "Type error! `init` is setting a field of the wrong type. It's trying to set a field of type `{}`, but the field it's setting actually has type `{}`.\n[{}]",
                    pretty::typ(&expected),
                    pretty::typ(&found),
                    pos
                )
            }
            TypeErrorTupleExpected(pos, op, t) => {
                println!(
                    "Type error! `{}` expects a tuple type, but it will receive a `{}` instead.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&t),
                    pos
                )
            }
            TypeErrorRegionHandleExpected(pos, op, t) => {
                println!(
                    "Type error! `{}` expects a region handle, but it will receive a `{}` instead.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&t),
                    pos
                )
            }
            TypeErrorFunctionExpected(pos, op, t) => {
                println!(
                    "Type error! `{}` expected a function, but it will receive a `{}` instead.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&t),
                    pos
                )
            }
            TypeErrorNonEmptyExistStack(pos) => {
                println!("Type error! At the end of the function there are still unbound existential variables.\n[{}]", pos)
            }
            TypeErrorNotEnoughCompileTimeArgs(pos, expected, found) => {
                println!("Type error! The function expects {} compile-time arguments, but only receives {}.\n[{}]", expected, found, pos)
            }
            TypeErrorNotEnoughRuntimeArgs(pos, expected, found) => {
                println!("Type error! The function expects {} arguments, but will only receive {}.\n[{}]", expected, found, pos)
            }
            TypeErrorCallArgTypesMismatch(pos, expected, found) => {
                println!(
                    "Type error! A function was called with the wrong types of arguments. It expects `{}`, but it's receiving `{}` instead.\n[{}]",
                    pretty::types(&expected),
                    pretty::types(&found),
                    pos
                )
            }
            CapabilityErrorBadInstantiation(pos, bound, present) => {
                println!(
                    "Capability error! A capability parameter is being instantiated with a capability that doesn't meet its bound requirement. The bound is `{}` but it's instantiated with `{}`.\n[{}]",
                    pretty::caps(&bound),
                    pretty::caps(&present),
                    pos
                )
            }
            KindErrorBadInstantiation(pos, kind, instantiation) => {
                println!(
                    "Kind error! A `{}` variable is being instantiated with a `{}`.\n[{}]",
                    pretty::kind(kind),
                    pretty::get_kind_str(&instantiation),
                    pos
                )
            }
            TypeError(pos, op, expected, found) => {
                println!(
                    "Type error! `{}` expected a `{}`, but it is receiving a `{}`.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&expected),
                    pretty::typ(&found),
                    pos
                )
            }
            TypeErrorNonEmptyKindContextOnMain => {
                println!("Type error! The entrypoint function cannot be polymorphic in any way.")
            }
            CapabilityErrorMainRequiresCapability => {
                println!("Capability error! The entrypoint function cannot require any capabilities.")
            }
            TypeErrorMainHasArgs => {
                println!("Type error! The entrypoint function cannot require arguments.")
            }
            RepresentationError(pos, op, expected, found) => {
                println!(
                    "Representation error! `{}` expected a `{}`, but it is receiving a `{}`.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::representation(&expected),
                    pretty::representation(&found),
                    pos
                )
            }
            RepresentationErrorBadInstantiation(pos, expected, found) => {
                println!(
                    "Representation error! A type variable with a `{}` representation is being instantiated with a type with a `{}` representation.\n[{}]",
                    pretty::representation(&expected),
                    pretty::representation(&found),
                    pos
                )
            }
            TypeErrorHandleExpected(pos, op, t) => {
                println!(
                    "Type error! `{}` expects a region handle, but it will receive a `{}` instead.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&t),
                    pos
                )
            }
            TypeErrorEmptyForallStack(pos, op) => {
                println!(
                    "Type error! `{}` is trying to use the forall stack, but at this point the forall stack has nothing in it.\n[{}]", 
                    pretty::unverified_op(op),
                    pos
                )
            }
            TypeErrorPolymorphicNonFunc(pos, op, t) => {
                println!(
                    "Type error! `{}` is trying to produce a polymorphic type, but only function types can be polymorphic, and it's receiving a non-function type `{}`.\n[{}]",
                    pretty::unverified_op(op),
                    pretty::typ(&t),
                    pos
                )
            }
            TypeErrorNonEmptyForallStack(label) => {
                println!("Type error! At the end of the function there are still unbound universal quantification variables.\n[{}]", label)
            }
            TypeErrorNonEmptyRgnVarStack(label) => {
                println!("Type error! At the end of the function there are still unbound region variables.\n[{}]", label)
            }
        },
        Ok(()) => (),
    }
}
