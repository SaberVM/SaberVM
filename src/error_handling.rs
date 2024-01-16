use crate::header::get_kind_str;
use crate::header::get_op_str;
use crate::header::pretty_caps;
use crate::header::pretty_kind;
use crate::header::pretty_t;
use crate::header::CapabilityPool;
use crate::header::Error;
use crate::header::Error::*;
use crate::header::TypeListPool;
use crate::header::TypePool;

pub fn handle(
    res: Result<(), Error>,
    cap_pool: &CapabilityPool,
    type_pool: &TypePool,
    tl_pool: &TypeListPool,
) {
    match res {
        Err(e) => {
            match e {
                SyntaxErrorParamNeeded(op) => {
                    println!(
                        "Syntax error! The file ended while a parameter for {} was expected!",
                        get_op_str(op)
                    )
                }
                SyntaxErrorUnknownOp(op) => {
                    println!("Syntax error! Unknown opcode {}!", op)
                }
                TypeErrorEmptyCTStack(op) => {
                    println!("Type error! {:#?} needs a value from the compile-time stack, but it was empty!", op)
                }
                KindErrorReq(val) => {
                    println!(
                        "Kind error! req needs a type or region, but it received a {}!",
                        get_kind_str(val)
                    )
                }
                KindError(op, expected, found) => {
                    println!(
                        "Kind error! {:#?} needs a {}, but it received a {}!",
                        op,
                        pretty_kind(expected),
                        get_kind_str(found)
                    )
                }
                TypeErrorEmptyExistStack(op) => {
                    println!("Type error! {:#?} needs a variable from the existential stack, but it was empty!", op)
                }
                TypeErrorParamOutOfRange(op) => {
                    println!("Type error! {:#?} has a parameter that's too large for what it's trying to index into!", op)
                }
                TypeErrorExistentialExpected(found) => {
                    println!(
                        "Type error! Expected an existential type, but found {}",
                        pretty_t(type_pool.get(found), &type_pool, &tl_pool, &cap_pool)
                    )
                }
                TypeErrorEmptyStack(op) => {
                    println!(
                        "Type error! {:#?} needs a something from the stack, but it's empty!",
                        op
                    )
                }
                CapabilityError(op, cr) => {
                    println!(
                        "Capability error! {:#?} doesn't have enough permission, only {}!",
                        op,
                        pretty_caps(cap_pool.get(cr))
                    )
                }
                TypeErrorInit(expected, found) => {
                    println!("Type error! init is setting a field of the wrong type! Expected {}, found {}", pretty_t(type_pool.get(expected), &type_pool, &tl_pool, &cap_pool), pretty_t(type_pool.get(found), &type_pool, &tl_pool, &cap_pool))
                }
                TypeErrorTupleExpected(op, tr) => {
                    println!("Type error! {:#?} expected a tuple type, but found a {} instead!", op, pretty_t(type_pool.get(tr), &type_pool, &tl_pool, &cap_pool))
                }
                TypeErrorRegionHandleExpected(op, tr) => {
                    println!("Type error! {:#?} expected a region handle, but found a {} instead!", op, pretty_t(type_pool.get(tr), &type_pool, &tl_pool, &cap_pool))
                }
                TypeErrorFunctionExpected(op, tr) => {
                    println!("Type error! {:#?} expected a function, but found a {} instead!", op, pretty_t(type_pool.get(tr), &type_pool, &tl_pool, &cap_pool))
                }
                TypeErrorNonEmptyExistStack() => {
                    println!("Type error! At the end of the function there are still unbound existential variables!")
                }
            }
        }
        Ok(()) => (),
    }
}
