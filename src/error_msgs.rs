/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::header::*;
use crate::pretty::Pretty;

pub fn msg(e: Error) -> String {
    match e {
        Error::SyntaxErrorParamNeeded(pos, op) => {
            format!("Syntax Error: Parameter needed for opcode {:?} at pos {}", op, pos)
        },
        Error::SyntaxErrorUnknownOp(pos, op) => {
            format!("Syntax Error: Unknown opcode {:?} at pos {}", op, pos)
        },
        Error::TypeErrorMainHasArgs => {
            format!("Type Error: Main function cannot have arguments")
        },
        Error::TypeErrorNonEmptyQuantificationStack(label) => {
            format!("Type Error: Non-empty quantification stack at label {}", label)
        },
        Error::TypeErrorEmptyQuantificationStack(pos, op) => {
            format!("Type Error: Empty quantification stack at pos {} for opcode {}", pos, op.pretty())
        },
        Error::TypeErrorEmptyCTStack(pos, op) => {
            format!("Type Error: Empty compile-time stack at pos {} for opcode {}", pos, op.pretty())
        },
        Error::TypeErrorEmptyStack(pos, op) => {
            format!("Type Error: Empty stack at pos {} for opcode {}", pos, op.pretty())
        },
        Error::KindError(pos, op, kind, ctval) => {
            format!("Kind Error: Expected {} at pos {} for opcode {} but found {}", kind.pretty(), pos, op.pretty(), ctval.kind().pretty())
        },
        Error::RegionError(pos, op, r1, r2) => {
            format!("Region Error: Expected region {} at pos {} for opcode {} but found {}", r1.pretty(), pos, op.pretty(), r2.pretty())
        },
        Error::TypeError(pos, op, t1, t2) => {
            format!("Type Error: Expected type {} at pos {} for opcode {} but found {}", t1.pretty(), pos, op.pretty(), t2.pretty())
        },
        Error::SizeError(pos, op, s1, s2) => {
            format!("Size Error: Expected size {} at pos {} for opcode {} but found {}", s1, pos, op.pretty(), s2)
        },
        Error::UniquenessError(pos, op, r) => {
            format!("Uniqueness Error: Expected unique region {} at pos {} for opcode {}", r.pretty(), pos, op.pretty())
        },
        Error::RegionAccessError(pos, op, r) => {
            format!("Region Access Error: Expected access to region {} at pos {} for opcode {}", r.pretty(), pos, op.pretty())
        },
        Error::TypeErrorSpecificTypeVarExpected(pos, op, id1, id2) => {
            format!("Type Error: Expected type variable a{} at pos {} for opcode {} but found a{}", id1.1, pos, op.pretty(), id2.1)
        },
        Error::TypeErrorTypeVarExpected(pos, op, id, t) => {
            format!("Type Error: Expected type variable a{} at pos {} for opcode {} but found {}", id.1, pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorCTGetOutOfRange(pos, i, max) => {
            format!("Type Error: ct_get out of range at pos {}: the compile-time stack depth is {} but got {}", pos, max, i)
        },
        Error::TypeErrorGetOutOfRange(pos, i, max) => {
            format!("Type Error: get out of range at pos {}: the stack depth is {} but got {}", pos, max, i)
        },
        Error::TypeErrorInitOutOfRange(pos, i, max) => {
            format!("Type Error: init out of range at pos {}: the stack depth is {} but got {}", pos, max, i)
        },
        Error::TypeErrorProjOutOfRange(pos, i, max) => {
            format!("Type Error: proj out of range at pos {}: the stack depth is {} but got {}", pos, max, i)
        },
        Error::TypeErrorExistentialExpected(pos, op, t) => {
            format!("Type Error: Expected existential type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorInitTypeMismatch(pos, t1, t2) => {
            format!("Type Error: Expected type {} at pos {} for init but found {}", t1.pretty(), pos, t2.pretty())
        },
        Error::TypeErrorTupleExpected(pos, op, t) => {
            format!("Type Error: Expected tuple type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorFunctionExpected(pos, op, t) => {
            format!("Type Error: Expected function type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorRegionHandleExpected(pos, op, t) => {
            format!("Type Error: Expected region handle type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorNotEnoughRuntimeArgs(pos, s1, s2) => {
            format!("Type Error: Not enough runtime arguments at pos {}: expected {} but got {}", pos, s1, s2)
        },
        Error::TypeErrorCallArgTypesMismatch(pos, ts1, ts2) => {
            format!("Type Error: Call argument types mismatch at pos {}: expected {} but got {}", pos, ts1.iter().map(|t| t.pretty()).collect::<Vec<_>>().join(", "), ts2.iter().map(|t| t.pretty()).collect::<Vec<_>>().join(", "))
        },
        Error::TypeErrorMallocNonTuple(pos, op, t) => {
            format!("Type Error: Expected tuple type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorPtrExpected(pos, op, t) => {
            format!("Type Error: Expected pointer type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorForallExpected(pos, op, t) => {
            format!("Type Error: Expected forall type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::TypeErrorForallRegionExpected(pos, op, t) => {
            format!("Type Error: Expected forall region type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::KindErrorBadApp(pos, op, ctval) => {
            format!("Kind Error: Expected type at pos {} for opcode {} but found {}", pos, op.pretty(), ctval.kind().pretty())
        },
        Error::TypeErrorDoubleInit(pos, op, n) => {
            format!("Type Error: Double init at pos {} for opcode {}: component {} has already been initialized", pos, op.pretty(), n)
        },
        Error::TypeErrorUninitializedRead(pos, op, n) => {
            format!("Type Error: Uninitialized read at pos {} for opcode {}: component {} has not been initialized", pos, op.pretty(), n)
        },
        Error::TooBigForStack(pos, op, t) => {
            format!("Type Error: Too big for stack at pos {} for opcode {}: {}", pos, op.pretty(), t.pretty())
        },
        Error::ForwardDeclNotType(t) => {
            format!("Forward declaration of non-type: {}", t.pretty())
        },
        Error::ForwardDeclRuntimeOp(op) => {
            format!("Forward declaration of runtime opcode: {}", op.pretty())
        },
        Error::ForwardDeclBadStack(ctvals) => {
            format!("Forward declaration of bad stack: {}", ctvals.iter().map(|ctval| ctval.kind().pretty()).collect::<Vec<_>>().join(", "))
        },
        Error::UnknownGlobalFunc(pos, op, label) => {
            format!("Unknown global function at pos {}, opcode {}: {}", pos, op.pretty(), label)
        },
        Error::UnexpectedEOF => {
            "Unexpected end of file".to_string()
        },
        Error::TypeErrorArrayExpected(pos, op, t) => {
            format!("Type Error: Expected array type at pos {} for opcode {} but found {}", pos, op.pretty(), t.pretty())
        },
        Error::ReadOnlyRegionError(pos, op, r) => {
            format!("Region Error: region is read-only at pos {} for opcode {}: {}", pos, op.pretty(), r.pretty())
        },
        Error::DataSectionLoadOutOfBounds(pos, op, loc, max) => {
            format!("Data section load out of bounds at pos {} for opcode {}: loading from {} but the data section ends at {}", pos, op.pretty(), loc, max)
        },
        Error::InvalidDataSectionType(pos, op, t) => {
            format!("Data section type error at pos {} for opcode {}: invalid data section type {}", pos, op.pretty(), t.pretty())
        },
        Error::CannotMutateDataSection(pos, op) => {
            format!("Data section mutation error at pos {} for opcode {}: cannot mutate data section", pos, op.pretty())
        },
    }
}