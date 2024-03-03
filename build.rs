fn main() {
    cc::Build::new()
            .file("src/vm.c")
            .compile("vm");
}