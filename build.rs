fn main() {
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=tdh");
    let _ = embed_resource::compile("assets/mdview.rc", embed_resource::NONE);
}
