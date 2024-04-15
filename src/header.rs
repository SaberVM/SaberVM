/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// The input type for SaberVM.
pub type ByteStream = Vec<u8>;

/// The output type for the parser.
pub type ParsedStmts = Vec<Stmt1>;

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
    Req,
    Region,
    Unique,
    Handle,
    I32,
    Tuple(u8),
    Quantify,
    Some,
    All,
    Rgn,
    End,
    Func(u8),
    CTGet(u8),
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
}

/// The type of unverified ops.
/// This includes all the static analysis ops, which disappear after verification.
#[derive(Clone, Copy, Debug)]
pub enum Op2 {
    Get(usize, usize),
    Init(usize, usize),
    Malloc(usize),
    Proj(usize, usize),
    Call,
    Print,
    Lit(i32),
    GlobalFunc(Label),
    Halt,
    NewRgn,
    FreeRgn,
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
    Tuple(Vec<Type>, Region),
    Var(Id, usize),
    Func(Vec<Type>),
    Forall(Id, usize, Box<Type>),
    ForallRegion(Region, Box<Type>),
    Exists(Id, usize, Box<Type>),
    Guess(Label),
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Self::I32 => 4,
            Self::Handle(_r) => 8,
            Self::Tuple(ts, _) => ts.iter().map(|t| t.size()).sum(),
            Self::Var(_id, s) => *s,
            Self::Func(_param_ts) => 8,
            Self::Forall(_id, _size, t) => t.size(),
            Self::ForallRegion(_r, t) => t.size(),
            Self::Exists(_id, _size, t) => t.size(),
            Self::Guess(_label) => panic!("size of guess type"),
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

pub enum KindContextEntry {
    Region(Region),
    Type(Id),
}

pub enum Quantification {
    Region(Region),
    Forall(Id, usize),
    Exist(Id, usize)
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
    TypeErrorMallocNonTuple(Pos, Op1, Type)
}
