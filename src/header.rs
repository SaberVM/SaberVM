/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// The input type for SaberVM.
pub type ByteStream = Vec<u8>; // not streamed currently

/// The output type for the parser.
pub type ParsedStmts = Vec<UnverifiedStmt>;

/// The type of parameters of parameterized opcodes (like `proj` or `get`).
pub type OpParam = u8;

/// The type of unverified ops.
/// This includes all the static analysis ops, which disappear after verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnverifiedOpcode {
    ReqOp,             // 0x00
    RegionOp,          // 0x01
    HeapOp,            // 0x02
    CapOp,             // 0x03
    CapLEOp,           // 0x04
    UniqueOp,          // 0x05
    RWOp,              // 0x06
    BothOp,            // 0x07
    HandleOp,          // 0x08
    I32Op,             // 0x09
    EndFunctionOp,     // 0x0A
    MutOp,             // 0x0B
    TupleOp(OpParam),  // 0x0C
    ArrOp,             // 0x0D
    AllOp,             // 0x0E
    SomeOp,            // 0x0F
    EmosOp,            // 0x10
    FuncOp(OpParam),   // 0x11
    CTGetOp(OpParam),  // 0x12
    CTPopOp,           // 0x13
    UnpackOp,          // 0x14
    GetOp(OpParam),    // 0x15
    InitOp(OpParam),   // 0x16
    MallocOp,          // 0x17
    ProjOp(OpParam),   // 0x18
    CallOp,            // 0x19
    PrintOp,           // 0x1A
    LitOp(i32),        // 0x1B
    GlobalFuncOp(u32), // 0x1C
    HaltOp(u8),        // 0x1D
    PackOp,            // 0x1E
    Word32Op,          // 0x1F
}

#[derive(Clone, Copy, Debug)]
pub enum VerifiedOpcode {
    GetOp(usize, usize),
    InitOp(usize),
    MallocOp(usize),
    ProjOp(usize),
    CallOp,
    PrintOp,
    LitOp(i32),
    GlobalFuncOp(Label),
    HaltOp(u8),
}

/// Statements produced by the parsing pass.
/// Next they would go through the verification pass.
#[derive(Debug)]
pub enum UnverifiedStmt {
    Func(Pos, Vec<UnverifiedOpcode>),
}

/// Statements produced by the verification pass.
#[derive(Debug)]
pub enum VerifiedStmt {
    Func(Pos, Type, Vec<VerifiedOpcode>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Repr {
    Word32Repr,
    Word64Repr,
    PtrRepr(Box<Repr>),
    TupleRepr(Vec<Repr>),
    ArrayRepr(Box<Repr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    RegionKind,
    TypeKind,
    CapabilityKind,
    ReprKind,
}

/// The type for identifiers.
/// As SaberVM is stack-based, this really just means compile-time stuff, like type variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub Pos, pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    VarRgn(Id),
    HeapRgn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    UniqueCap(Region),
    ReadWriteCap(Region),
    VarCap(Id),
}

/// The type for things that can be in the kind context (\Delta, in the Capability Calculus paper) of a function.
/// Capability variables get bounds recorded here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindContextEntry {
    CapabilityKindContextEntry(Id, Vec<Capability>),
    TypeKindContextEntry(Id, Repr),
    RegionKindContextEntry(Id),
}

/// The kind assignments for polymorphism variables for a function.
/// Capability variables also have bounds recorded here.
/// This is \Delta in the Capability Calculus paper.
pub type KindContext = Vec<KindContextEntry>;

pub type Label = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    I32Type,
    HandleType(Region),
    MutableType(Box<Type>),
    TupleType(Vec<Type>, Region),
    ArrayType(Box<Type>, Region),
    VarType(Id, Repr),
    FuncType(KindContext, Vec<Capability>, Vec<Type>),
    ExistsType(Id, Repr, Box<Type>),
    GuessType(Label),
}

pub fn get_repr(t: &Type) -> Repr {
    match t {
        Type::I32Type => Repr::Word32Repr,
        Type::HandleType(_) => Repr::Word64Repr,
        Type::MutableType(t) => get_repr(t),
        Type::TupleType(ts, _) => Repr::PtrRepr(Box::new(Repr::TupleRepr(ts.iter().map(get_repr).collect()))),
        Type::ArrayType(t, _) => Repr::PtrRepr(Box::new(Repr::ArrayRepr(Box::new(get_repr(t))))),
        Type::VarType(_, r) => r.clone(),
        Type::FuncType(_, _, _) => Repr::Word32Repr,
        Type::ExistsType(_, _, t) => get_repr(&*t),
        Type::GuessType(_) => panic!("tried to get repr of GuessType"),
    }
}

/// The type of things on the compile-time stack, which can come in any kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTStackVal {
    RegionCTStackVal(Region),
    CapCTStackVal(Vec<Capability>),
    TypeCTStackVal(Type),
    ReprCTStackVal(Repr),
}

/// Get the kind of a compile-time stack value.
pub fn get_kind(ctval: &CTStackVal) -> Kind {
    match ctval {
        CTStackVal::CapCTStackVal(_) => Kind::CapabilityKind,
        CTStackVal::RegionCTStackVal(_) => Kind::RegionKind,
        CTStackVal::TypeCTStackVal(_) => Kind::TypeKind,
        CTStackVal::ReprCTStackVal(_) => Kind::ReprKind,
    }
}

pub type Pos = u32;

/// a do-nothing type wrapper for annotating error arguments
pub type Expected<A> = A;
/// a do-nothign type wrapper for annotating error arguments
pub type Found<A> = A;

/// The type for user-facing errors (as opposed to internal SaberVM errors, which are panics).
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    SyntaxErrorParamNeeded(Pos, u8),
    SyntaxErrorUnknownOp(Pos, u8),
    TypeErrorEmptyCTStack(Pos, UnverifiedOpcode),
    KindErrorReq(Pos, CTStackVal),
    KindError(Pos, UnverifiedOpcode, Kind, CTStackVal),
    TypeErrorEmptyExistStack(Pos, UnverifiedOpcode),
    TypeErrorParamOutOfRange(Pos, UnverifiedOpcode),
    TypeErrorExistentialExpected(Pos, Type),
    TypeErrorEmptyStack(Pos, UnverifiedOpcode),
    CapabilityError(
        Pos,
        UnverifiedOpcode,
        Expected<Vec<Capability>>,
        Found<Vec<Capability>>,
    ),
    TypeErrorInit(Pos, Expected<Type>, Found<Type>),
    TypeErrorTupleExpected(Pos, UnverifiedOpcode, Type),
    TypeErrorRegionHandleExpected(Pos, UnverifiedOpcode, Type),
    TypeErrorFunctionExpected(Pos, UnverifiedOpcode, Type),
    TypeErrorNonEmptyExistStack(Pos),
    TypeErrorNotEnoughCompileTimeArgs(Pos, Expected<usize>, Found<usize>),
    TypeErrorNotEnoughRuntimeArgs(Pos, Expected<usize>, Found<usize>),
    TypeErrorCallArgTypesMismatch(Pos, Expected<Vec<Type>>, Found<Vec<Type>>),
    CapabilityErrorBadInstantiation(Pos, Expected<Vec<Capability>>, Found<Vec<Capability>>),
    KindErrorBadInstantiation(Pos, Kind, CTStackVal),
    TypeError(Pos, UnverifiedOpcode, Expected<Type>, Found<Type>),
    TypeErrorNonEmptyKindContextOnMain,
    CapabilityErrorMainRequiresCapability,
    TypeErrorMainHasArgs,
    RepresentationError(Pos, UnverifiedOpcode, Expected<Repr>, Found<Repr>),
    RepresentationErrorBadInstantiation(Pos, Expected<Repr>, Found<Repr>),
}
