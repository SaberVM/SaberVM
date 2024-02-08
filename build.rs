fn main() {
    cc::Build::new()
        .object("src/vm.o")
        .compile("vm");
}