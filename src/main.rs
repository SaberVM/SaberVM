mod header;
mod parse;
mod verify;

use std::fs;

use crate::header::pretty_op2;
use crate::header::pretty_t;
use crate::header::CapabilityPool;
use crate::header::Stmt2::*;
use crate::header::TypeListPool;
use crate::header::TypePool;

fn main() {
    let code = fs::read("bin.svm").unwrap();
    let prog = parse::go(&code);
    match prog {
        Err(n) => {
            dbg!(n);
        }
        Ok(prog) => {
            let mut cap_pool = CapabilityPool(vec![]);
            let mut type_pool = TypePool(vec![]);
            let mut tl_pool = TypeListPool(vec![]);

            let prog2 = verify::go(prog, &mut cap_pool, &mut type_pool, &mut tl_pool);
            match prog2 {
                Ok(stmts) => {
                    let p = &stmts[0];
                    let Func2(_, t, ops) = p;
                    dbg!(pretty_t(&t, &type_pool, &tl_pool, &cap_pool));
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

// I keep this here for documentation:
// let code: Vec<u8> = vec![
//     0x00,   // _op_        _run-time_stack_    _compile-time_stack_
//     0x00,
//     0x00,
//     0x00,
//     0x01,   // region                           r
//     0x12,   // ct_get 0                         r,r
//     0x00,
//     0x06,   // read                             {+r},r
//     0x04,   // cap_le                           {c≤+r},r
//     0x12,   // ct_get 0                         {c≤+r},{c≤+r},r
//     0x00,
//     0x00,   // req                              {c≤+r},r
//     0x0E,   // all                              a,{c≤+r},r
//     0x12,   // ct_get 0                         a,a,{c≤+r},r
//     0x00,
//     0x00,   // req          a                   a,{c≤+r},r
//     0x12,   // ct_get 2     a                   r,a,{c≤+r},r
//     0x02,
//     0x0F,   // some         a                   b,r,a,{c≤+r},r
//     0x12,   // ct_get 3     a                   {c≤+r},b,r,a,{c≤+r},r
//     0x03,
//     0x12,   // ct_get 5     a                   r,{c≤+r},b,r,a,{c≤+r},r
//     0x05,
//     0x12,   // ct_get 4     a                   a,r,{c≤+r},b,r,a,{c≤+r},r
//     0x04,
//     0x12,   // ct_get 0     a                   a,a,r,{c≤+r},b,r,a,{c≤+r},r
//     0x00,
//     0x0C,   // tuple 2      a                   a*a@r,{c≤+r},b,r,a,{c≤+r},r
//     0x02,
//     0x12,   // ct_get 2     a                   b,a*a@r,{c≤+r},b,r,a,{c≤+r},r
//     0x02,
//     0x11,   // func 2       a                   [{c≤+r}](b,a*a@r),b,r,a,{c≤+r},r
//     0x02,
//     0x0C,   // tuple 2      a                   [{c≤+r}](b,a*a@r)*b@r,a,{c≤+r},r
//     0x02,
//     0x10,   // emos         a                   some b.[{c≤+r}](b,a*a@r)*b@r,a,{c≤+r},r
//     0x00,   // req          a,k                 a,{c≤+r},r
//     0x12,   // ct_get 2     a,k                 r,a,{c≤+r},r
//     0x02,
//     0x08,   // handle       a,k                 handle(r),a,{c≤+r},r
//     0x00,   // req          a,k,r               a,{c≤+r},r
//     0x12,   // ct_get 2     a,k,r               r,a,{c≤+r},r
//     0x02,
//     0x12,   // ct_get 1     a,k,r               a,r,a,{c≤+r},r
//     0x01,
//     0x12,   // ct_get 0     a,k,r               a,a,r,a,{c≤+r},r
//     0x00,
//     0x0C,   // tuple 2      a,k,r               a*a@r,a,{c≤+r},r
//     0x02,
//     0x15,   // get 2        r,a,k,r             a*a@r,a,{c≤+r},r
//     0x02,
//     0x17,   // malloc       (_,_),a,k,r         a,{c≤+r},r
//     0x15,   // get 1        a,(_,_),a,k,r       a,{c≤+r},r
//     0x01,
//     0x16,   // init 0       (a,_),a,k,r         a,{c≤+r},r
//     0x01,
//     0x15,   // get 1        a,(a,_),a,k,r       a,{c≤+r},r
//     0x01,
//     0x16,   // init 1       (a,a),a,k,r         a,{c≤+r},r
//     0x01,
//     0x15,   // get 2        k,(a,a),a,k,r       a,{c≤+r},r
//     0x02,
//     0x14,   // unpack       (f,c),(a,a),a,k,r   a,{c≤+r},r
//     0x18,   // proj 1       c,(a,a),a,k,r       a,{c≤+r},r
//     0x01,
//     0x15,   // get 3        k,c,(a,a),a,k,r     a,{c≤+r},r
//     0x03,
//     0x14,   // unpack       (f,c),c,(a,a),a,k,r a,{c≤+r},r
//     0x18,   // proj 0       f,c,(a,a),a,k,r     a,{c≤+r},r
//     0x00,
//     0x19,   // clean 3      f,c,(a,a)           a,{c≤+r},r
//     0x03,
//     0x1A    // call         --                  --
// ];
