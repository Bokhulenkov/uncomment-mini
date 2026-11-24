fn main() {
    let parser = "src/parser.c";
    cc::Build::new().std("c11").include("src").file(parser).compile("tree-sitter-objc");
    println!("cargo:rerun-if-changed={parser}");
}
