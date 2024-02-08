rm -r target/debug/build
(
    cd src
    clang -g -c -Wl,-U,_main vm.c
    cargo build
)