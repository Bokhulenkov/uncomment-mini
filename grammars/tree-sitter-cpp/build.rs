fn main() {
    let parser = "src/parser.c";
    let scanner = "src/scanner.c";
    cc::Build::new()
        .std("c11")
        .include("src")
        .file(parser)
        .file(scanner)
        .compile("tree-sitter-cpp");
    println!("cargo:rerun-if-changed={parser}");
    println!("cargo:rerun-if-changed={scanner}");
}
