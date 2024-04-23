# SaberVM

The Saber Virtual Machine is a VM intended to be openly targeted by functional language compilers. It will eventually have a number of implementations on different systems, and it will hopefully be easy enough to implement your own on your system/platform of choice.

### What is SaberVM?

SaberVM uses a typed stack-based bytecode language with compile-time and run-time instructions. Each instruction is one byte, though some instructions eat the next 1-4 bytes as a compile-time parameter. This all leads to a very expressive compile-time language for annotations that SaberVM uses for safety guarantees and, eventually, optimization.

By design, SaberVM will be able to be AOT-compiled with a number of optimizations, run on a JIT VM, or run on a simple VM for hobby projects or resource-constrained environments. Currently, a naive VM implementation is fairly fleshed-out, and being used as a prototype for rapid design iteration.

SaberVM has some of the resilience guarantees of the BEAM, and uses a concurrent semantics for being run in parallel or sequentially. It can also be compiled without memory safety runtime checks/tagging for a more LLVM-like experience. SaberVM is built on the research of [Dr. Greg Morrisett](https://scholar.google.com/citations?user=Dswus94AAAAJ&hl=en), some of which (see Cyclone in [here](https://doc.rust-lang.org/reference/influences.html)) inspired Rust's memory safety model.

In a traditional CPS-based compiler pipeline, SaberVM is targeted after closure-conversion and hoisting, though like Erlang it will eventually have higher-level languages (namely Saber) to target and even use for projects.

### Why target SaberVM?

A common choice for compiler backends is LLVM. However, for CPS-based compilers of functional languages, LLVM is not really an option. CPS is a good backend language for functional languages, for reasons that are too low-level to be expressed in LLVM's IR. LLVM is too tied to having C-style calling conventions and a return-address stack. In addition, functional languages (with pervasive immutability and recursion) can have their own set of good optimizations and heuristics that LLVM won't do. SaberVM aims to fill the vacuum left by LLVM, by being a CPS-based compiler backend oriented towards functional languages. However, it doesn't expect to be the only player in that field, and therefore makes opinionated tradeoffs. Namely, SaberVM tries to optimize for safety, portability, and expressivity.

SaberVM bytecode can guarantee memory safety and deadlock freedom, without a loss of expressivity. This is achieved in a way similar to [Vale](https://vale.dev/), with a combination of borrow-checker-like compiler analysis and generational-reference-like runtime checks. These runtime checks don't crash the program when a dangling pointer is dereferenced, they instead put the program in a built-in panic mode so the broken parts of the program can be restarted, as in Erlang (but more low-level and manual). 

This all combines to no loss in expressivity for safe programs, such that you can even write garbage collectors for your language without borrow-checker-induced nightmares. Supporting user-written garbage collectors is, of course, a hard requirement when we're trying to support functional languages. SaberVM's memory model finds a way to do this without compromising the soundness of the memory safety guarantees.

However, a compiler that cares more about performance than safety can turn off the runtime checks and tagging without any change to the bytecode; SaberVM's bytecode language is designed to be runtime-check-agnostic: proper resource handling is checked, not inferred.

SaberVM can also be used in a JVM-style, where the compiler simply outputs SaberVM bytecode and ships it to users, who have their own SaberVM or AOT-compiler. Thus they can run your program in a safe mode if they don't trust you (like websites), or a fast mode if they do (like games).

### How does it work?

SaberVM has two main systems at play: regions and exceptions. 

#### Regions

A region can be thought of as an arena with a malloc/free-style internal memory management system. To read or write from the heap, you need a region. Regions have statically-checked lifetimes and a borrow-checker-like system for checking that values within that region are only read or written to during the lifetime of the region. When a region's lifetime ends, it is freed like an arena. Doing borrow-checking on the scale of regions instead of values means that values can be freely mutated and aliased however you see fit; the system ensures that pointers into the region are never aliased after the region is freed, even if they continue to be passed around.

This structure of memory is important because in a safe compilation the values in the heap are tagged with information about which inhabitant is at that place in memory, so pointers can check that they're pointing at the thing they think they are when they're dereferenced. This introduces a memory fragmentation issue: "slots" in memory can then later be used only by values that are the same size or smaller; they have an unchangeable "max size." To prove this, consider two values **adjacent** in memory, `A` and `B`, and their pointers, `&A` and `&B`. Now say we free **both**, and allocate `C` at the same address where `A` was (that is, `&C == &A`). If `C` is bigger than `A` was, then it has arbitrary control of the bytes used to tag `B`! `B` has been freed, but now that `C` controls those bytes, it could potentially make the bytes where `B`'s tag was appear to still show `B` as unfreed. That means a nefarious program could potentially cause `&B` to think that `B` is still there, leading dereferences of `&B` to read memory controlled by `C`, instead of panicking.

The solution to this is to require that `C` is no larger than the `A` that was there before it. If allocating something in memory fixes a certain max-size for that bit of memory for the rest of the program's lifetime, that can cause issues with poor use of memory. Therefore, SaberVM puts its values in **regions** so there are certain points in the program's lifecycle where it's statically known that nothing will dereference some set of pointers ever again, so their referent memory can be _really_ freed, with no restriction on its future use. As a language writer, if you find your output programs have significant fragmentation issues, you can do some light region inference to fix it. In addition, since regions are freed like arenas (reading uninitialized memory is statically prevented), regions offer a way to deallocate a bunch of memory instantly, and improve cache locality.

#### Exceptions

NOTE: this is currently unimplemented in the VM prototype, simply because I'm just one person. You can [contribute](CONTRIBUTING.md) these things yourself (super appreciated!) or financially support me so I can work fewer hours at my day job, if you want to see these features sooner.

SaberVM's other interesting system is exceptions. Exceptions in SaberVM are not like normal exceptions, though there's nothing stopping a compiler writer from building a normal exception system on top of SaberVM. Instead, SaberVM exceptions **don't take arguments**. Every function must have a catch-all exception case, and only that. Why? Having this built-in to SaberVM means that instructions that fail at runtime don't crash your program, they just jump to the exception handler. The language targeting SaberVM is then expected to produce exception handlers that do at least one of four things:

 1. crash the program (with an explicit `halt` instruction)
 2. rethrow the exception (that is, jump to the _caller's_ catch-all exception handler)
 3. restart the crashed function (in a microreboot or Erlang style, without information about what caused the exception)
 4. release held resources (currently SaberVM doesn't have locks, only CAS, but this is likely to change)

**Note that SaberVM exceptions are not expected to be how your own language handles its exceptions!**

For example, if you prefer a `Result`-style exception handling, you can write functions that attempt single fallible instructions with an exception handler that produces the corresponding `Err` value.

### The progress so far

Currently work is underway on an MVP, that is, a simple non-JITing VM in Rust. Like Wasm, SaberVM bytecode must be verified before it is run. The project so far can parse an array of bytes, typecheck it, and execute it, but only supports a subset of the full SaberVM design. Thankfully, SaberVM has been designed to be easy to implement, so that languages targeting it can easily extend the number of platforms they support. Indeed, development has gone extremely swiftly so far.

The type checker uses a two-phase system: functions are forward-declared, and then their definitions are checked. This can be done in parallel trivially, though it isn't at the moment.

There are 27 instructions implemented right now, chosen to implement a simple "duplicate" function in naive CPS.

```
foo x = (x, x)
```

In SaberVM this has a fairly complex type:

```
Forall r: Rgn, t: Type: 8-byte. (t, Exists u: 8-byte. ((u, (t, t)@r)->0, u)@r, handle(r))->0
```

To pick this apart a little, the function is quantified over a region, and a type that must be 8 bytes in size. `(a..)->0` is the syntax for a function type that takes the arguments `a..`. The `->0` notation is to suggest that it's a CPS function, and therefore doesn't return. With the default region-quantification, you can imagine that `r` is borrowed, if you're familiar with Rust. `Exists x: repr. t` is an existential type, and is mostly used for typing closure environments. In this case, the continuation is existentially quantified so that it could have any closure environment struct. That is, a closure is typed at `Exists x. ((x, a..)->0, x)@R`. The `(a..)@R` notation is product types (tuples) that are allocated in region `R`. Finally, `handle(R)` is the singleton type for the runtime information of a region `R`, holding for example the pointer to the physical region in memory.

In summary, that type says that the function takes a value of type `t`, a region `r`, and a continuation that expects a value of type `(t, t)` allocated in region `r`. It also requires that `r` hasn't been freed. That should seem like a reasonable type for the duplicate function. The richness of the information present in the type should hopefully justify how many compile-time instructions are used to construct it! Resorting too much to type inference would hurt the performance of the verifier asymptotically, so in the tradeoff between binary size and verifier performance I chose a faster verifier. Very small instructions helps the binary size a lot already.

I'm also writing a little lambda-calculus [compiler](https://github.com/RyanBrewer317/SaberLC) to get much better tests of SaberVM.

This is very WIP; the purpose of this MVP is not to ship it per se, but mostly to play with the system and figure out what breaking changes need to be made. I've already gone through several major rewrites and redesigns!

Polymorphism is currently just done in a simple Kinds-Are-Calling-Conventions-inspired approach, where the number of bytes of acceptible are specified by the quantifier. This is like given SaberVM a second, much simpler, C-like type system that has no polymorphism and only cares about numbers of bytes.

### $25+/month Sponsors!

[<img src="https://github.com/SimonAndCarmen.png" width="60px;"/><br /><sub><a href="https://github.com/SimonAndCarmen">SimonAndCarmen</a></sub>](https://github.com/SimonAndCarmen)

[<img src="https://github.com/emberian.png" width="60px;"/><br /><sub><a href="https://github.com/emberian">emberian</a></sub>](https://github.com/emberian)
