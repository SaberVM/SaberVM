# Contributing to SaberVM

If you're considering contributing to SaberVM, you're in the right place.
Thank you so much for your interest and time!
This project is very eager for contributions :)

Before proceeding, please note our [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting Bugs

A bug could be something breaking, like a crash or an incorrect result, or it could just be some inconvenience, like a gap in documentation. If you don't know if something's a bug, assume that it is for now, and we can figure it out together.

Contributing bug reports is a great way to get started with contributing to open source! If you think you've found a bug, first take a moment to check if someone's already reported it, using [this list](https://github.com/RyanBrewer317/SaberVM-Text-Lang/issues). If you don't see anything, that means that you get to create a new issue! This is extremely helpful to our project. You can use [this link](https://github.com/RyanBrewer317/SaberVM-Text-Lang/issues/new).

When you are creating a bug report, please be **as detailed as possible**. Contributors who are trying to tackle the issue can only help if they're able to reproduce the behavior you're seeing, so they'll need information about the machine you're using, the operating system, your coding software, etc.

## Improving Our Code

If you're excited about this project and want to help out, there are a number of things you can do. One thing is tests, mentioned more below. If instead you want to really add to the functionality of SaberVM, there's a section on how to build from source (that is, get to the point where you can start adding things). Following that are sections on the design tradeoffs, direction, and philosophy of the project, to get a sense of what contributions we will love, and which ones we'll have to reject.

### Tests

An easy and very valuable way to start contributing to SaberVM is to add some tests! 
We definitely need way more. 
Check out the end of [parse.rs](src/parse.rs) for a good example of how to add a new test.

### Building From Source

Here's a little section about getting the SaberVM codebase to run on your machine.

First you'll need Rust and a C compiler. Once you've got those, you can build SaberVM with `cargo build`.

For Windows, you'll need to use the MSVC toolchain. For example, the C compiler might be called `cl` instead of `clang` or `gcc`.

For rapid development, I typically use `cargo run` which builds the project and also immediately runs the executable.

SaberVM currently tries to run the `bin.svm` file in the repository. If you want it to run something else instead, overwrite `bin.svm` with the binary file you want to run. Note that the first four bytes should be `0x00`. If you know the text instructions you want to run but don't want to go through the effort of making a binary file with that, you can try [this project](https://github.com/RyanBrewer317/SaberVM-Text-Lang) for generating the binary file, though it's often not quite up to date.

### Project Organization

Currently, each file in `src` holds a separate part of the project. That is, we don't have separate directories for these things. SaberVM is intended to be small and portable by design.

[`header.rs`](src/header.rs) contains top-level definitions that the rest of the rust code will need. This is the types for the AST, the types and other static analysis things, the errors SaberVM might run into in the case of bad input (for example, type errors). Pretty-printing for all of these things is defined in [`pretty.rs`](src/pretty.rs).

[`main.rs`](src/main.rs) is the entrypoint. It reads the `bin.svm` file and handles the passing of information into the [parser](src/parse.rs), then to the [verifier](src/verify.rs), and finally to the [VM](src/vm.rs). If any errors crop up during this process, they get immediately handed to [`error_handling.rs`](src/error_handling.rs).

The VM is made up of two files, in two languages. [`vm.rs`](src/vm.rs) takes the verified AST, collapses it into a byte array, and hands it to [`vm.c`](src/vm.c), which performs the final execution.

### Design Direction and Philosophy

SaberVM is at the intersection of a ton of design constraints, which severely limits the direction of the project. 

One important one is **portability**, that is, making it as easy as possible to implement this whole thing yourself. Portability is extremely important for community-oriented software, where people feel like they can make it themselves instead of trusting foreign software. It also helps adoption, since it becomes very easy to add more platforms that support running SaberVM bytecode. 

This means a fair amount of the other constraints ought to be reached via advanced computer science theory. Why? Because it's only *building* the thing which has to be really easy. Understanding why it works it is allowed to be a bit harder, since it's not necessary for making it. Therefore theory is something for which we have a bigger budget.

Another constraint is **safety**. VMs like SaberVM are much more useful if they can be made safe, as this is one of the reasons people use VMs instead of running code natively. The JVM is used to run untrusted software by constraining memory access and using garbage collection, so all memory usage is safe. The WebAssembly VM is used to run untrusted code by sandboxing the runtime, though it has no safety for its own memory. 

SaberVM achieves memory safety without sandboxing or garbage collection, using static analysis. The theory could be explained with category theory using comonads, though it's much simpler to just explain it in normal words. The idea is simple: when you do something with memory in your function, the function must be annotated with the fact that it needs that memory to be accessable. For example, it shouldn't be possible to free the memory and then call that function that touches it. These constraints bubble up: if your function `fun1` calls another function `fun2`, then the annotations of `fun1` have to satisfy the annotations of `fun2`. Fun, right? :) These annotations are called "capabilities," because they grant the *capability* to access the memory. I wrote more about the idea (and its original paper) in much more detail [here](https://ryanbrewer.dev/posts/safe-mmm-with-coeffects.html).

