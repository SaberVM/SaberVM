
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
    Func1(i32, Vec<OpCode1>),
}

#[derive(Debug)]
pub enum Stmt2 {
    Func2(i32, TypeRef, Vec<OpCode2>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    KRegion,
    KType,
    KCapability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    RegionVar(Id),
    Heap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    Owned(Region),
    NotOwned(Region),
    CapVar(Id),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindContextEntry {
    KCEntryCapability(Id, Vec<Capability>),
    KCEntryType(Id),
    KCEntryRegion(Id),
}

pub type KindContext = Vec<KindContextEntry>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Ti32,
    THandle(Region),
    TMutable(TypeRef),
    TTuple(TypeListRef, Region),
    TArray(TypeRef),
    TVar(Id),
    TFunc(KindContext, Vec<Capability>, TypeListRef),
    TExists(Id, TypeRef),
    TGuess(i32),
}

pub struct TypePool(pub Vec<Type>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeRef(u32);
impl TypePool {
    pub fn get(&self, r: &TypeRef) -> &Type {
        let TypePool(v) = self;
        let TypeRef(i) = r;
        &v[*i as usize]
    }
    pub fn add(&mut self, t: Type) -> TypeRef {
        let TypePool(v) = self;
        let idx = v.len();
        v.push(t);
        TypeRef(idx.try_into().expect("too many types in the pool"))
    }
}

pub struct TypeListPool(pub Vec<Vec<TypeRef>>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeListRef(u32);
impl TypeListPool {
    pub fn get(&self, r: &TypeListRef) -> &Vec<TypeRef> {
        let TypeListPool(v) = self;
        let TypeListRef(i) = r;
        &v[*i as usize]
    }
    pub fn add(&mut self, ts: Vec<TypeRef>) -> TypeListRef {
        let TypeListPool(v) = self;
        let idx = v.len();
        v.push(ts);
        TypeListRef(idx.try_into().expect("too many type lists in the pool"))
    }
}

#[derive(Clone, Debug)]
pub enum CTStackVal {
    CTRegion(Region),
    CTCapability(Vec<Capability>),
    CTType(TypeRef),
}

pub fn get_kind(ctval: &CTStackVal) -> Kind {
    match ctval {
        CTStackVal::CTCapability(_) => Kind::KCapability,
        CTStackVal::CTRegion(_) => Kind::KRegion,
        CTStackVal::CTType(_) => Kind::KType
    }
}

#[derive(Debug)]
pub enum Error {
    SyntaxErrorParamNeeded(u8),
    SyntaxErrorUnknownOp(u8),
    TypeErrorEmptyCTStack(OpCode1),
    KindErrorReq(CTStackVal),
    KindError(OpCode1, Kind, CTStackVal),
    TypeErrorEmptyExistStack(OpCode1),
    TypeErrorParamOutOfRange(OpCode1),
    TypeErrorExistentialExpected(TypeRef),
    TypeErrorEmptyStack(OpCode1),
    CapabilityError(OpCode1, Vec<Capability>),
    TypeErrorInit(TypeRef, TypeRef),
    TypeErrorTupleExpected(OpCode1, TypeRef),
    TypeErrorRegionHandleExpected(OpCode1, TypeRef),
    TypeErrorFunctionExpected(OpCode1, TypeRef),
    TypeErrorNonEmptyExistStack,
    ErrorTodo
}
