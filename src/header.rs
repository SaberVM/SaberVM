#[derive(Clone, Copy, Debug)]
pub enum OpCode1 {
    Op1Req(),     // 0x00
    Op1Region(),  // 0x01
    Op1Heap(),    // 0x02
    Op1Cap(),     // 0x03
    Op1CapLE(),   // 0x04
    Op1Own(),     // 0x05
    Op1Read(),    // 0x06
    Op1Both(),    // 0x07
    Op1Handle(),  // 0x08
    Op1i32(),     // 0x09
    Op1End(),     // 0x0A
    Op1Mut(),     // 0x0B
    Op1Tuple(u8), // 0x0C
    Op1Arr(),     // 0x0D
    Op1All(),     // 0x0E
    Op1Some(),    // 0x0F
    Op1Emos(),    // 0x10
    Op1Func(u8),  // 0x11
    Op1CTGet(u8), // 0x12
    Op1CTPop(),   // 0x13
    Op1Unpack(),  // 0x14
    Op1Get(u8),   // 0x15
    Op1Init(u8),  // 0x16
    Op1Malloc(),  // 0x17
    Op1Proj(u8),  // 0x18
    Op1Clean(u8), // 0x19
    Op1Call(),    // 0x1A
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

pub fn pretty_op2(op: &OpCode2) -> String {
    match op {
        OpCode2::Op2Get(n) => "get ".to_owned() + &n.to_string(),
        OpCode2::Op2Init(n) => "init ".to_owned() + &n.to_string(),
        OpCode2::Op2Malloc(_n) => "malloc".to_owned(),
        OpCode2::Op2Proj(n) => "proj ".to_owned() + &n.to_string(),
        OpCode2::Op2Clean(n, _m) => "clean ".to_owned() + &n.to_string(),
        OpCode2::Op2Call() => "call".to_owned(),
    }
}

#[derive(Debug)]
pub enum Stmt1 {
    Func1(i32, Vec<OpCode1>),
}

#[derive(Debug)]
pub enum Stmt2 {
    Func2(i32, Type, Vec<OpCode2>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    KRegion,
    KType,
    KCapability(Option<CapabilityRef>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    RegionVar(Id),
    Heap(),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Owned(Region),
    NotOwned(Region),
    CapVar(Id),
    CapVarBounded(Id, CapabilityRef),
}

pub struct CapabilityPool(pub Vec<Vec<Capability>>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Ti32(),
    THandle(Region),
    TMutable(TypeRef),
    TTuple(TypeListRef, Region),
    TArray(TypeRef),
    TVar(Id),
    TForall(Id, Kind, TypeRef),
    TExists(Id, TypeRef),
    TFunc(CapabilityRef, TypeListRef),
    TGuess(i32)
}

pub fn pretty_region(r: Region) -> String {
    match r {
        Region::Heap() => "Heap".to_string(),
        Region::RegionVar(id) => "r".to_owned() + &id.1.to_string(),
    }
}

pub fn pretty_caps(cs: &Vec<Capability>) -> String {
    "{".to_owned()
        + &cs
            .iter()
            .map(|c| pretty_cap(*c))
            .collect::<Vec<_>>()
            .join(",")
        + "}"
}

pub fn pretty_cap(c: Capability) -> String {
    match c {
        Capability::CapVar(id) => "c".to_owned() + &id.1.to_string(),
        Capability::CapVarBounded(id, _cr) => "c".to_owned() + &id.1.to_string(),
        Capability::Owned(r) => "1".to_owned() + &pretty_region(r),
        Capability::NotOwned(r) => "+".to_owned() + &pretty_region(r),
    }
}

pub fn pretty_ts(
    ts: &TypeListRef,
    type_pool: &TypePool,
    tl_pool: &TypeListPool,
    cap_pool: &CapabilityPool,
) -> String {
    tl_pool
        .get(*ts)
        .iter()
        .map(|tr| pretty_t(type_pool.get(*tr), type_pool, tl_pool, cap_pool))
        .collect::<Vec<_>>()
        .join(", ")
}

fn var(id: &Id, k: Kind, cap_pool: &CapabilityPool) -> String {
    match k {
        Kind::KRegion => "r".to_owned() + &id.1.to_string(),
        Kind::KType => "t".to_owned() + &id.1.to_string(),
        Kind::KCapability(None) => "c".to_owned() + &id.1.to_string(),
        Kind::KCapability(Some(c)) => {
            "c".to_owned() + &id.1.to_string() + "≤" + &pretty_caps(cap_pool.get(c))
        }
    }
}

pub fn pretty_t(
    t: &Type,
    type_pool: &TypePool,
    tl_pool: &TypeListPool,
    cap_pool: &CapabilityPool,
) -> String {
    match t {
        Type::Ti32() => "i32".to_string(),
        Type::THandle(r) => "handle(".to_owned() + &pretty_region(*r) + ")",
        Type::TMutable(tr) => {
            "mut ".to_owned() + &pretty_t(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        Type::TTuple(ts, r) => {
            "(".to_owned()
                + &pretty_ts(ts, type_pool, tl_pool, cap_pool)
                + ")@"
                + &pretty_region(*r)
        }
        Type::TArray(tr) => {
            "[]".to_owned() + &pretty_t(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        Type::TVar(id) => "t".to_owned() + &id.1.to_string(),
        Type::TForall(id, k, tr) => {
            "Forall ".to_owned()
                + &var(&id, *k, cap_pool)
                + ". "
                + &pretty_t(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        Type::TExists(id, tr) => {
            "Exists t".to_owned()
                + &id.1.to_string()
                + ". "
                + &pretty_t(type_pool.get(*tr), type_pool, tl_pool, cap_pool)
        }
        Type::TFunc(c, ts) => {
            "[".to_owned()
                + &pretty_caps(cap_pool.get(*c))
                + "]("
                + &pretty_ts(ts, type_pool, tl_pool, cap_pool)
                + ")->0"
        },
        Type::TGuess(_) => panic!("type-checking artifact lasted too long")
    }
}

// pub enum ActualType {
//     ATi32(),
//     ATHandle(Region),
//     ATMutable(Box<ActualType>),
//     ATTuple(Vec<ActualType>, Region),
//     ATArray(Box<ActualType>),
//     ATVar(Id),
//     ATForall(Id, Box<ActualType>),
//     ATExists(Id, Box<ActualType>),
//     ATFunc(CapabilityRef, Vec<ActualType>),
// }

// pub fn actualize(t: TypeRef, type_pool: &TypePool, tl_pool: &TypeListPool) -> ActualType {
//     match type_pool.get(t) {
//         Type::Ti32() => ActualType::ATi32(),
//         Type::THandle(r) => ActualType::ATHandle(*r),
//         Type::TMutable(tr) => ActualType::ATMutable(Box::new(actualize(*tr, &type_pool, &tl_pool))),
//         Type::TTuple(tsr, r) => {
//             let mut ts = vec![];
//             for tr in tl_pool.get(*tsr) {
//                 ts.push(actualize(*tr, &type_pool, &tl_pool))
//             }
//             ActualType::ATTuple(ts, *r)
//         }
//         Type::TArray(tr) => ActualType::ATArray(Box::new(actualize(*tr, &type_pool, &tl_pool))),
//         Type::TVar(id) => ActualType::ATVar(*id),
//         Type::TForall(id, tr) => {
//             ActualType::ATForall(*id, Box::new(actualize(*tr, &type_pool, &tl_pool)))
//         }
//         Type::TExists(id, tr) => {
//             ActualType::ATExists(*id, Box::new(actualize(*tr, &type_pool, &tl_pool)))
//         }
//         Type::TFunc(c, tsr) => {
//             let mut ts = vec![];
//             for tr in tl_pool.get(*tsr) {
//                 ts.push(actualize(*tr, &type_pool, &tl_pool))
//             }
//             ActualType::ATFunc(*c, ts)
//         }
//     }
// }

// pub fn substitute(old: Id, new: Id, t: ActualType) -> ActualType {
//     match t {
//         ActualType::ATVar(id) => {
//             if id == old {
//                 ActualType::ATVar(new)
//             } else {
//                 t
//             }
//         }
//         ActualType::ATi32() | ActualType::ATHandle(_) => t,
//         ActualType::ATMutable(tr) => ActualType::ATMutable(Box::new(substitute(old, new, *tr))),
//         ActualType::ATArray(tr) => ActualType::ATArray(Box::new(substitute(old, new, *tr))),
//         ActualType::ATTuple(ts, r) => {
//             let mut ts2 = vec![];
//             for t in ts {
//                 ts2.push(substitute(old, new, t))
//             }
//             ActualType::ATTuple(ts2, r)
//         }
//         ActualType::ATForall(id, tr2) => {
//             if id == old {
//                 t
//             } else {
//                 ActualType::ATForall(id, Box::new(substitute(old, new, *tr2)))
//             }
//         }
//         ActualType::ATExists(id, tr2) => {
//             if id == old {
//                 t
//             } else {
//                 ActualType::ATExists(id, Box::new(substitute(old, new, *tr2)))
//             }
//         }
//         ActualType::ATFunc(c, ts) => {
//             let mut ts2 = vec![];
//             for t in ts {
//                 ts2.push(substitute(old, new, t))
//             }
//             ActualType::ATFunc(c, ts2)
//         }
//     }
// }

pub struct TypePool(pub Vec<Type>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
