/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// The input type for SaberVM.
pub type ByteStream = Vec<u8>;

pub type Pos = u32;
pub type Label = u32;

/// The type for identifiers.
/// As SaberVM is stack-based, this really just means compile-time stuff, like type variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub Pos, pub u32);

/// The type of unverified ops.
/// This includes all the static analysis ops, which disappear after verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op1 {
    Unique,
    Handle,
    I32,
    Tuple(u8),
    Some,
    All,
    Rgn,
    End,
    App,
    Func(u8),
    CTGet(u8),
    Lced,
    Unpack,
    Get(u8),
    Init(u8),
    Malloc,
    Proj(u8),
    Call,
    Print,
    Lit(i32),
    GlobalFunc(u32),
    Halt,
    Pack,
    Size(u32),
    NewRgn,
    FreeRgn,
    Ptr,
    Deref,
}

/// The type of unverified ops.
/// This includes all the static analysis ops, which disappear after verification.
#[derive(Clone, Copy, Debug)]
pub enum Op2 {
    Get(usize, usize),
    Init(usize, usize, usize),
    InitIP(usize, usize),
    Malloc(usize),
    Alloca(usize),
    Proj(usize, usize, usize),
    ProjIP(usize, usize),
    Call,
    Print,
    Lit(i32),
    GlobalFunc(Label),
    Halt,
    NewRgn,
    FreeRgn,
    Deref(usize),
}

#[derive(Debug)]
pub enum ForwardDec {
    Func(Pos, Vec<Op1>),
}

/// Statements produced by the parsing pass.
/// Next they would go through the verification pass.
#[derive(Debug)]
pub enum Stmt1 {
    Func(Pos, Vec<Op1>),
}

/// Statements produced by the verification pass.
#[derive(Debug)]
pub enum Stmt2 {
    Func(Pos, Type, Vec<Op2>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub unique: bool,
    pub id: Id,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    I32,
    Handle(Region),
    Tuple(Vec<(bool, Type)>),
    Ptr(Box<Type>, Region),
    Var(Id, usize),
    Func(Vec<Type>),
    Forall(Id, usize, Box<Type>),
    ForallRegion(Region, Box<Type>, Vec<Region>),
    Exists(Id, usize, Box<Type>),
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Self::I32 => 4,
            Self::Handle(_r) => 8,
            Self::Tuple(ts) => ts.iter().map(|(_, t)| t.size()).sum(),
            Self::Ptr(_t, _r) => 16,
            Self::Var(_id, s) => *s,
            Self::Func(_param_ts) => 4,
            Self::Forall(_id, _size, t) => t.size(),
            Self::ForallRegion(_r, t, _captured_rgns) => t.size(),
            Self::Exists(_id, _size, t) => t.size(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Region,
    Type,
    Size,
}

/// The type of things on the compile-time stack, which can come in any kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTStackVal {
    Region(Region),
    Type(Type),
    Size(usize),
}

impl CTStackVal {
    // pub fn kind(&self) -> Kind {
    //     match self {
    //         Self::Region(_) => Kind::Region,
    //         Self::Type(_) => Kind::Type,
    //         Self::Size(_) => Kind::Size,
    //     }
    // }
}

#[derive(Debug)]
pub enum Quantification {
    Region(Region),
    Forall(Id, usize),
    Exist(Id, usize),
}

/// The type for user-facing errors (as opposed to internal SaberVM errors, which are panics).
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    SyntaxErrorParamNeeded(Pos, u8),
    SyntaxErrorUnknownOp(Pos, u8),
    TypeErrorMainHasArgs,
    TypeErrorNonEmptyQuantificationStack(Label),
    TypeErrorEmptyQuantificationStack(Pos, Op1),
    TypeErrorEmptyCTStack(Pos, Op1),
    TypeErrorEmptyStack(Pos, Op1),
    KindError(Pos, Op1, Kind, CTStackVal),
    RegionError(Pos, Op1, Region, Region),
    TypeError(Pos, Op1, Type, Type),
    SizeError(Pos, Op1, usize, usize),
    UniquenessError(Pos, Op1, Region),
    RegionAccessError(Pos, Op1, Region),
    TypeErrorSpecificTypeVarExpected(Pos, Op1, Id, Id),
    TypeErrorTypeVarExpected(Pos, Op1, Id, Type),
    TypeErrorCTGetOutOfRange(Pos, u8, usize),
    TypeErrorGetOutOfRange(Pos, u8, usize),
    TypeErrorInitOutOfRange(Pos, u8, usize),
    TypeErrorProjOutOfRange(Pos, u8, usize),
    TypeErrorExistentialExpected(Pos, Op1, Type),
    TypeErrorInitTypeMismatch(Pos, Type, Type),
    TypeErrorTupleExpected(Pos, Op1, Type),
    TypeErrorFunctionExpected(Pos, Op1, Type),
    TypeErrorRegionHandleExpected(Pos, Op1, Type),
    TypeErrorNotEnoughRuntimeArgs(Pos, usize, usize),
    TypeErrorCallArgTypesMismatch(Pos, Vec<Type>, Vec<Type>),
    TypeErrorMallocNonTuple(Pos, Op1, Type),
    TypeErrorPtrExpected(Pos, Op1, Type),
    TypeErrorForallExpected(Pos, Op1, Type),
    TypeErrorForallRegionExpected(Pos, Op1, Type),
    KindErrorBadApp(Pos, Op1, CTStackVal),
    TypeErrorDoubleInit(Pos, Op1, u8),
    TypeErrorUninitializedRead(Pos, Op1, u8)
}
