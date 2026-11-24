fn main() {
    let parser = "src/parser.c";
    cc::Build::new().std("c11").include("src").file(parser).compile("tree-sitter-c");
    println!("cargo:rerun-if-changed={parser}");
}
