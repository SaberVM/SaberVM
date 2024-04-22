fn main() {
    cc::Build::new()
        .file("src/vm.h")
        .file("src/vm.c")
        .compile("vm");
}
