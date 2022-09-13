extern crate cxx_build;

fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/demo.cpp")
        .flag_if_supported("-std=c++17")
        .compile("cxxbridge-demo");

    cc::Build::new()
        .file("src/demo_c.c")
        .define("FOO", Some("bar"))
        .include("src")
        .compile("demo_c");

    println!("cargo:rustc-link-lib=demo_c");

    println!("cargo:rerun-if-changed=src/demo_c.c");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/demo.h");
    println!("cargo:rerun-if-changed=src/demo.cpp");
}
