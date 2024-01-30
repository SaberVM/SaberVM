
#[derive(Clone, Copy, Debug)]
pub enum OpCode1 {
    Op1Req,     // 0x00
    Op1Region,  // 0x01
    Op1Heap,    // 0x02
    Op1Cap,     // 0x03
    Op1CapLE,   // 0x04
    Op1Own,     // 0x05
    Op1Read,    // 0x06
    Op1Both,    // 0x07
    Op1Handle,  // 0x08
    Op1i32,     // 0x09
    Op1End,     // 0x0A
    Op1Mut,     // 0x0B
    Op1Tuple(u8), // 0x0C
    Op1Arr,     // 0x0D
    Op1All,     // 0x0E
    Op1Some,    // 0x0F
    Op1Emos,    // 0x10
    Op1Func(u8),  // 0x11
    Op1CTGet(u8), // 0x12
    Op1CTPop,   // 0x13
    Op1Unpack,  // 0x14
    Op1Get(u8),   // 0x15
    Op1Init(u8),  // 0x16
    Op1Malloc,  // 0x17
    Op1Proj(u8),  // 0x18
    Op1Call,    // 0x19
}

#[derive(Clone, Copy, Debug)]
pub enum OpCode2 {
    Op2Get(u8),
    Op2Init(u8),
    Op2Malloc(u8),
    Op2Proj(u8),
    Op2Call,
}

#[derive(Debug)]
pub enum Stmt1 {
    Func1(Pos, Vec<OpCode1>),
}

#[derive(Debug)]
pub enum Stmt2 {
    Func2(Pos, Type, Vec<OpCode2>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    KRegion,
    KType,
    KCapability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub Pos, pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    RegionVar(Id),
    Heap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    Unique(Region),
    ReadWrite(Region),
    CapVar(Id),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindContextEntry {
    KCEntryCapability(Id, Vec<Capability>, Capability),
    KCEntryType(Id, Type),
    KCEntryRegion(Id, Region),
}

pub type KindContext = Vec<KindContextEntry>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Ti32,
    THandle(Region),
    TMutable(Box<Type>),
    TTuple(Vec<Type>, Region),
    TArray(Box<Type>),
    TVar(Id),
    TFunc(KindContext, Vec<Capability>, Vec<Type>),
    TExists(Id, Box<Type>),
    TGuess(i32),
}

#[derive(Clone, Debug)]
pub enum CTStackVal {
    CTRegion(Region),
    CTCapability(Vec<Capability>),
    CTType(Type),
}

pub fn get_kind(ctval: &CTStackVal) -> Kind {
    match ctval {
        CTStackVal::CTCapability(_) => Kind::KCapability,
        CTStackVal::CTRegion(_) => Kind::KRegion,
        CTStackVal::CTType(_) => Kind::KType
    }
}

pub type Pos = u32;

#[derive(Debug)]
pub enum Error {
    SyntaxErrorParamNeeded(Pos, u8),
    SyntaxErrorUnknownOp(Pos, u8),
    TypeErrorEmptyCTStack(Pos, OpCode1),
    KindErrorReq(Pos, CTStackVal),
    KindError(Pos, OpCode1, Kind, CTStackVal),
    TypeErrorEmptyExistStack(Pos, OpCode1),
    TypeErrorParamOutOfRange(Pos, OpCode1),
    TypeErrorExistentialExpected(Pos, Type),
    TypeErrorEmptyStack(Pos, OpCode1),
    CapabilityError(Pos, OpCode1, Vec<Capability>, Vec<Capability>), // expected, found
    TypeErrorInit(Pos, Type, Type), // expected, found
    TypeErrorTupleExpected(Pos, OpCode1, Type),
    TypeErrorRegionHandleExpected(Pos, OpCode1, Type),
    TypeErrorFunctionExpected(Pos, OpCode1, Type),
    TypeErrorNonEmptyExistStack(Pos),
    TypeErrorNotEnoughCompileTimeArgs(Pos, usize, usize), // expected, found
    TypeErrorNotEnoughRuntimeArgs(Pos, usize, usize), // expected, found
    TypeErrorCallArgTypesMismatch(Pos, Vec<Type>, Vec<Type>), // expected, found
    CapabilityErrorBadInstantiation(Pos, Vec<Capability>, Vec<Capability>), // expected, found
    KindErrorBadInstantiation(Pos, Kind, CTStackVal),
}
