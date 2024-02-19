fn main() {
    cc::Build::new()
        .file("libsmacker-src/smacker.c")
        .file("libsmacker-src/smk_bitstream.c")
        .file("libsmacker-src/smk_hufftree.c")
        .compile("libsmacker");
}
