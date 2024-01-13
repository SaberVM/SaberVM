mod header;
mod parse;
mod verify;

use crate::header::CapabilityPool;
use crate::header::TypeListPool;
use crate::header::TypePool;
use crate::header::Stmt2::*;
use crate::header::pretty_op2;

fn main() {
    let code: Vec<u8> = vec![
        0x00, 
        0x00, 
        0x00, 
        0x00, 
        0x01,   // region               r
        0x12,   // ct_get 0             r,r
        0x00,
        0x06,   // read                 {+r},r
        0x04,   // cap_le               {c≤+r},r
        0x12,   // ct_get 0             {c≤+r},{c≤+r},r
        0x00,
        0x00,   // req                  {c≤+r},r
        0x0E,   // all                  a,{c≤+r},r
        0x12,   // ct_get 0             a,a,{c≤+r},r
        0x00,
        0x00,   // req          a       a,{c≤+r},r
        0x12,   // ct_get 2     a       r,a,{c≤+r},r
        0x02,
        0x0F,   // some         a       b,r,a,{c≤+r},r
        0x12,   // ct_get 3     a       {c≤+r},b,r,a,{c≤+r},r
        0x03,
        0x12,   // ct_get 5     a       r,{c≤+r},b,r,a,{c≤+r},r
        0x05,
        0x12,   // ct_get 4     a       a,r,{c≤+r},b,r,a,{c≤+r},r
        0x04,
        0x12,   // ct_get 0     a       a,a,r,{c≤+r},b,r,a,{c≤+r},r
        0x00,
        0x0C,   // tuple 2      a       a*a@r,{c≤+r},b,r,a,{c≤+r},r
        0x02,
        0x12,   // ct_get 2     a       b,a*a@r,{c≤+r},b,r,a,{c≤+r},r
        0x02,
        0x11,   // func 2       a       [{c≤+r}](b,a*a@r),b,r,a,{c≤+r},r
        0x02,
        0x0C,   // tuple 2      a       [{c≤+r}](b,a*a@r)*b@r,a,{c≤+r},r
        0x02,
        0x10,   // emos         a       some b.[{c≤+r}](b,a*a@r)*b@r,a,{c≤+r},r
        0x00,   // req          a,k     a,{c≤+r},r
        0x12,   // ct_get 2     a,k     r,a,{c≤+r},r
        0x02,
        0x08,   // handle       a,k     handle(r),a,{c≤+r},r
        0x00,   // req          a,k,r   a,{c≤+r},r
        0x12,   // ct_get 2     a,k,r   r,a,{c≤+r},r
        0x02,
        0x12,   // ct_get 1     a,k,r   a,r,a,{c≤+r},r
        0x01,
        0x12,   // ct_get 0     a,k,r   a,a,r,a,{c≤+r},r
        0x00,
        0x0C,   // tuple 2      a,k,r   a*a@r,a,{c≤+r},r
        0x02,
        0x15,   // get 2        r,a,k,r a*a@r,a,{c≤+r},r
        0x02,
        0x17,   // malloc
        0x15,   // get 1
        0x01,
        0x16,   // init 0
        0x01,
        0x15,   // get 1
        0x01,
        0x16,   // init 1
        0x01,
        0x15,   // get 2
        0x02,
        0x14,   // unpack
        0x18,   // proj 1
        0x01,
        0x15,   // get 3
        0x03,
        0x14,   // unpack
        0x18,   // proj 0
        0x00,
        0x19,   // clean 3
        0x03,
        0x1A    // call
    ];
    let prog = parse::go(&code);
    match prog {
        Err(n) => {
            dbg!(n);
        }
        Ok(prog) => {
            let cap_pool = CapabilityPool(vec![]);
            let type_pool = TypePool(vec![]);
            let tl_pool = TypeListPool(vec![]);
        
            let prog2 = verify::first_pass(&prog[0], cap_pool, type_pool, tl_pool);
            match prog2 {
                Ok(p) => {
                    let Func2(_, _, ops) = p;
                    for op in ops {
                        println!("{}", pretty_op2(&op))
                    }
                }
                Err(e) => {
                    println!("Error!");
                    dbg!(e);
                }
            }
        }
    }
}

                // vec![
                //     Op1Region(),
                //     Op1Cap(),
                //     Op1CTGet(0),
                //     Op1Req(),
                //     Op1All(),
                //     Op1CTGet(0),
                //     Op1Req(),
                //     Op1CTGet(2),
                //     Op1Some(),
                //     Op1CTGet(3),
                //     Op1CTGet(5),
                //     Op1CTGet(4),
                //     Op1CTGet(0),
                //     Op1Tuple(2),
                //     Op1CTGet(2),
                //     Op1Func(2),
                //     Op1Tuple(2),
                //     Op1Emos(),
                //     Op1Req(),
                //     Op1CTGet(2),
                //     Op1Handle(),
                //     Op1Req(),
                //     Op1CTGet(2),
                //     Op1CTGet(1),
                //     Op1CTGet(0),
                //     Op1Tuple(2),
                //     Op1Get(2),
                //     Op1Malloc(),
                //     Op1Get(1),
                //     Op1Init(0),
                //     Op1Get(1),
                //     Op1Init(1),
                //     Op1Get(2),
                //     Op1Unpack(),
                //     Op1Proj(1),
                //     Op1Get(3),
                //     Op1Unpack(),
                //     Op1Proj(0),
                //     Op1Clean(3),
                //     Op1Call(),
                // ],