

#[derive(Clone, Copy, Debug)]
pub enum OpCode1 {
  Op1Req(),
  Op1Region(),
  Op1Heap(),
  Op1Cap(),
  Op1CapLE(),
  Op1Own(),
  Op1Read(),
  Op1Both(),
  Op1Handle(),
  Op1i32(),
  Op1End(),
  Op1Mut(),
  Op1Tuple(u8),
  Op1Arr(),
  Op1All(),
  Op1Some(),
  Op1Emos(),
  Op1Func(u8),
  Op1CTGet(u8),
  Op1CTPop(),
  Op1Get(u8),
  Op1Init(u8),
  Op1Malloc(),
  Op1Proj(u8),
  Op1Clean(u8),
  Op1Call(),
}

#[derive(Clone, Copy, Debug)]
pub enum OpCode2 {
  Op2Get(u8),
  Op2Init(u8),
  Op2Malloc(u8),
  Op2Proj(u8),
  Op2Clean(u8, u8),
  Op2Call(),
}

#[derive(Debug)]
pub enum Stmt1 {
  Func1(i32, Vec<OpCode1>),
}

#[derive(Debug)]
pub enum Stmt2 {
  Func2(i32, Vec<OpCode2>),
}

#[derive(Clone, Copy, Debug)]
pub struct Id(pub i32, pub i32);

#[derive(Clone, Copy, Debug)]
pub enum Region {
  RegionVar(Id),
  Heap(),
}

#[derive(Clone, Copy, Debug)]
pub enum Capability {
  Owned(Region),
  ReadOnly(Region),
  CapVar(Id),
  CapVarBounded(Id, CapabilityRef),
}

pub struct CapabilityPool(pub Vec<Vec<Capability>>);
#[derive(Clone, Copy, Debug)]
pub struct CapabilityRef(u32);
impl CapabilityPool {
  pub fn get(&self, r: CapabilityRef) -> &Vec<Capability> {
    let CapabilityPool(v) = self;
    let CapabilityRef(i) = r;
    &v[i as usize]
  }
  pub fn add(&mut self, cap: Vec<Capability>) -> CapabilityRef {
    let CapabilityPool(v) = self;
    let idx = v.len();
    v.push(cap);
    CapabilityRef(idx.try_into().expect("too many capabilities in the pool"))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
  Ti32(),
  THandle(Region),
  TMutable(TypeRef),
  TTuple(TypeListRef, Region),
  TArray(TypeRef),
  TVar(Id),
  TForall(Id, TypeRef),
  TExists(Id, TypeRef),
  TFunc(CapabilityRef, TypeListRef),
}

pub struct TypePool(pub Vec<Type>);
#[derive(Clone, Copy, Debug)]
pub struct TypeRef(u32);
impl TypePool {
  pub fn get(&self, r: TypeRef) -> &Type {
    let TypePool(v) = self;
    let TypeRef(i) = r;
    &v[i as usize]
  }
  pub fn add(&mut self, t: Type) -> TypeRef {
    let TypePool(v) = self;
    let idx = v.len();
    v.push(t);
    TypeRef(idx.try_into().expect("too many types in the pool"))
  }
}

pub struct TypeListPool(pub Vec<Vec<TypeRef>>);
#[derive(Clone, Copy, Debug)]
pub struct TypeListRef(u32);
impl TypeListPool {
  pub fn get(&self, r: TypeListRef) -> &Vec<TypeRef> {
    let TypeListPool(v) = self;
    let TypeListRef(i) = r;
    &v[i as usize]
  }
  pub fn add(&mut self, ts: Vec<TypeRef>) -> TypeListRef {
    let TypeListPool(v) = self;
    let idx = v.len();
    v.push(ts);
    TypeListRef(idx.try_into().expect("too many type lists in the pool"))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum CTStackVal {
  CTRegion(Region),
  CTCapability(CapabilityRef),
  CTType(TypeRef),
}
