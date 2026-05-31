fn main() {
    uniffi::generate_scaffolding("src/simplecomic.udl").unwrap();
    println!("cargo:rerun-if-changed=src/simplecomic.udl");
}
