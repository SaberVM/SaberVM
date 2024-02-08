# SaberVM

The Saber Virtual Machine is a VM intended to be openly targeted by functional language compilers. It will eventually have a number of implementations on different systems, and it will hopefully be easy enough to implement your own on your system/platform of choice.

### What is SaberVM?

SaberVM uses a typed stack-based bytecode language with compile-time and run-time instructions. Each instruction is one byte, though some instructions eat the next byte as a compile-time parameter. This all leads to a very expressive compile-time language for annotations that SaberVM will use for safety guarantees and optimization.

By design, SaberVM can be AOT-compiled with a number of optimizations, run on a JIT VM, or run on a simple VM for hobby projects or resource-constrained environments. It has some of the resilience guarantees of the BEAM, and uses a concurrent semantics for being run in parallel or sequentially. It can also be compiled without memory safety runtime checks/tagging for a more LLVM-like experience. SaberVM is built on the research of [Dr. Greg Morrisett](https://scholar.google.com/citations?user=Dswus94AAAAJ&hl=en), some of which (see Cyclone in [here](https://doc.rust-lang.org/reference/influences.html)) inspired Rust's memory safety model.

SaberVM is targeted after closure-conversion and hoisting in a traditional CPS pipeline, though like Erlang it will eventually have higher-level languages (namely Saber) to target and even use for projects.

### Why target SaberVM?

A common choice for compiler backends is LLVM. However, for CPS-based compilers of functional languages, LLVM is not really an option. CPS is a good backend language for functional languages, for reasons that are too low-level to be expressed in LLVM's IR. LLVM is too tied to having a C-style calling convention and a return-address stack. In addition, functional languages (with pervasive immutability and recursion) can have their own set good optimizations and heuristics that LLVM won't do. SaberVM aims to fill the hole left by LLVM, by being a CPS-based compiler backend oriented towards functional languages. However, it doesn't expect to be the only player in that field, and is therefore tries to be really good at some things at the cost of others. Namely, SaberVM tries to optimize for safety, portability, and expressivity.

SaberVM bytecode can guarantee memory safety and deadlock freedom, without a loss of expressivity. This is achieved in a way similar to [Vale](https://vale.dev/), with a combination of borrow-checker-like compiler analysis and generational-reference-like runtime checks. These runtime checks don't crash the program when a dangling pointer is dereferenced, they instead put the program in a built-in panic mode so the broken parts of the program can be restarted, as in Erlang (but more low-level and do-it-yourself). 

This all combines to no loss in expressivity for safe programs, such that you can even write garbage collectors for your language without borrow-checker-induced nightmares. This is of course a necessary feature of a VM for functional languages.

However, a compiler that cares more about performance than safety can turn off the runtime checks and tagging without any change to the bytecode; SaberVM bytecode is designed to be runtime-check-agnostic.

SaberVM can also be used in a JVM-style, where the compiler simply outputs SaberVM bytecode and ships it to users, who have their own SaberVM or AOT-compiler. Thus they can run your program in a safe mode if they don't trust you, or a fast mode if they do.

### How does it work?

SaberVM has two main systems at play: regions and exceptions. 

#### Regions

A region can be thought of as an arena with a malloc/free-style internal memory management system. To read or write from the heap, you need a region. It can be the heap itself. Regions have statically-checked lifetimes, and a [capability](https://dl.acm.org/doi/10.1145/292540.292564)-based system for checking that values within that region are only read or written to during the lifetime of the region. When a region's lifetime ends, it is freed like an arena.

This structure of memory is important because in a safe compilation the values in the heap are tagged with information about which inhabitant is at that place in memory, so pointers can check that they're pointing at the thing they think they are when they're dereferenced. This introduces a memory fragmentation issue: "slots" in memory can then later be used only by values that are the same size or smaller; they have an unchangeable "max size." To prove this, consider two values **adjacent** in memory, `A` and `B`, and their pointers, `&A` and `&B`. Now say we free both, and allocate `C` at the same address where `A` was (that is, `&C == &A`). If `C` is bigger than `A` was, then it has arbitrary control of the bytes used to tag `B`! That means a nefarious program could potentially cause `&B` to think that `B` is still there, leading dereferences to not crash but instead to read memory controlled by `C`.

If allocating something in memory fixes a certain max-size for that bit of memory for the rest of the program's lifetime, that can cause issues with poor use of memory. Therefore, SaberVM puts its values in **regions** so there are certain points where it's statically known that nothing will dereference some set of pointers ever again, so their referent memory can be _really_ freed, with no restriction on its future use. As a language writer, if you find your output programs have significant fragmentation issues, you can do some light region inference to fix it. In addition, since regions are freed like arenas (reading uninitialized memory is statically prevented), regions offer a way to deallocate a bunch of memory instantly, and improve cache locality.

#### Exceptions

SaberVM's other interesting system is exceptions. Exceptions in SaberVM are not like normal exceptions, though there's nothing stopping a compiler writer from building a normal exception system on top of SaberVM. Instead, SaberVM exceptions **don't take arguments**. Every function must have a catch-all exception case, and only that. Why? Having this built-in to SaberVM means that instructions that fail at runtime don't crash your program, they just jump to the exception handler. The language targeting SaberVM is then expected to produce exception handlers that do at least one of four things:

 1. crash the program (with an explicit `halt` instruction)
 2. rethrow the exception (that is, jump to the _caller's_ catch-all exception handler)
 3. restart the crashed function (in a microreboot or Erlang style, without information about what caused the exception)
 4. release held resources (currently SaberVM doesn't have locks, only CAS, but this is likely to change)

**Note that SaberVM exceptions are not expected to be how your own language handles its exceptions!**

For example, if you prefer a `Result`-style exception handling, you can write functions that attempt single fallible instructions with an exception handler that produces the corresponding `Err` value.

### The progress so far

Currently work is underway on an MVP, that is, a simple non-JITing VM in Rust. Like Wasm, SaberVM bytecode must be verified before it is run. The project so far can parse an array of bytes, typecheck it, and execute it, but only supports a small subset of the full SaberVM design. Thankfully, SaberVM has been designed to be easy to implement, so that languages targeting it can easily extend the number of platforms they support. Indeed, design took about a month but development has gone extremely swiftly so far, with about a week of full-time work for one person so far (but it wasn't actually a contiguous week).

The type checker uses a two-pass system exploiting the structure of CPS code. The first pass can determine the type of the function without knowing the types of any other functions, allowing trivial parallelism. The second pass uses the types of all the functions to verify the calls at the end of the functions. This design is easy to implement and very fast, though I haven't parallelized it yet because it's just an MVP.

There are 36 instructions implemented so far, which I believe includes all the compile-time instructions of the MVP (21). The only 11 runtime instructions were chosen so I could execute a simple "duplicate" function. In a high-level language this might look like so:

```
foo x = (x, x)
```

In SaberVM this is about 70 bytes of one- and two-byte instructions, excluding 4-byte literals, mostly because this function has a very rich type:

```
Forall r: Rgn, c≤{+r}: Cap, t: Type. [c](t, Exists u. ([c](u, (t, t)@r)->0, u)@r, handle(r))->0
```

To pick this apart a little, the function is quantified over a region, a capability that must at least be able to read and write into the region, and a type. `[C](a..)->0` is the syntax for a function type that can only be called when capability `C` is satisfied, and it takes the arguments `a..`. The `->0` notation is to suggest that it's a CPS function, and therefore doesn't return. `{+r}` is a capability to use pointers into region `r`, but it doesn't grant the ability to free `r`. You can imagine that `r` is borrowed, if you're familiar with Rust. `Exists x. t` is an existential type, and is mostly used for typing closure environments. In this case, the continuation is existentially quantified so that it could have any closure environment struct. That is, a closure is typed at `Exists x. ([C](x, a..)->0, x)@R`. The `(a..)@R` notation is product types (tuples) that are allocated in region `R`. Finally, `handle(R)` is the singleton type for the runtime information of a region `R`, holding for example the pointer to the physical region in memory.

In summary, that type says that the function takes a value of type `t`, a region `r`, and a continuation that expects a value of type `(t, t)` allocated in region `r`. It also requires that `r` hasn't been freed. That should seem like a reasonable type for the duplicate function. The richness of the information present in the type should hopefully justify how many compile-time instructions are used to construct it. Resorting too much to type inference would hurt the performance of the verifier asymptotically, so in the tradeoff between binary size and verifier performance I chose a faster verifier. One- and two-byte instructions helps the binary size a lot already.

To see how writing these 70ish bytes looks in practice, go to [example.md](https://github.com/RyanBrewer317/SaberVM/blob/main/example.md) for the example. Note that it also outputs the instructions the VM will execute, with the compile-time stuff all erased. To turn `example.md` into bytecode, I have another project [here](https://github.com/RyanBrewer317/SaberVM-Text-Lang). SaberVM can't quite execute this right now because of a bug in the memory management instructions, but programs that don't use malloc/init/proj instructions run great.

This is very WIP; the purpose of this MVP is not to ship it per se, but mostly to play with the system and figure out what breaking changes need to be made. 

Polymorphism is currently just done in a simple Kinds-Are-Calling-Conventions approach.

### Sponsors

[<img src="https://github.com/SimonAndCarmen.png" width="60px;"/><br /><sub><a href="https://github.com/SimonAndCarmen">SimonAndCarmen</a></sub>](https://github.com/SimonAndCarmen)

[<img src="https://github.com/emberian.png" width="60px;"/><br /><sub><a href="https://github.com/emberian">emberian</a></sub>](https://github.com/emberian)
