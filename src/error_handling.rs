
use crate::header::Error;
use crate::header::Error::*;
use crate::pretty;

pub fn handle(
    res: Result<(), Error>,
) {
    match res {
        Err(e) => match e {
            SyntaxErrorParamNeeded(op) => {
                println!(
                    "Syntax error! The file ended while a parameter for {} was expected!",
                    pretty::op_u8(op)
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
                    pretty::get_kind_str(&val)
                )
            }
            KindError(op, expected, found) => {
                println!(
                    "Kind error! {:#?} needs a {}, but it received a {}!",
                    op,
                    pretty::kind(expected),
                    pretty::get_kind_str(&found)
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
                    pretty::typ(&found)
                )
            }
            TypeErrorEmptyStack(op) => {
                println!(
                    "Type error! {:#?} needs a something from the stack, but it's empty!",
                    op
                )
            }
            CapabilityError(op, cs) => {
                println!(
                    "Capability error! {:#?} doesn't have enough permission, only {}!",
                    op,
                    pretty::caps(&cs)
                )
            }
            TypeErrorInit(expected, found) => {
                println!(
                    "Type error! init is setting a field of the wrong type! Expected {}, found {}",
                    pretty::typ(&expected),
                    pretty::typ(&found)
                )
            }
            TypeErrorTupleExpected(op, t) => {
                println!(
                    "Type error! {:#?} expected a tuple type, but found a {} instead!",
                    op,
                    pretty::typ(&t)
                )
            }
            TypeErrorRegionHandleExpected(op, t) => {
                println!(
                    "Type error! {:#?} expected a region handle, but found a {} instead!",
                    op,
                    pretty::typ(&t)
                )
            }
            TypeErrorFunctionExpected(op, t) => {
                println!(
                    "Type error! {:#?} expected a function, but found a {} instead!",
                    op,
                    pretty::typ(&t)
                )
            }
            TypeErrorNonEmptyExistStack => {
                println!("Type error! At the end of the function there are still unbound existential variables!")
            }
            ErrorTodo(s) => println!("{}", s)
        },
        Ok(()) => (),
    }
}
