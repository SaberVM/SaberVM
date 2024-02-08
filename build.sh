rm -r target/debug/build
clang -c -Wl,-U,_main vm.c
cargo run