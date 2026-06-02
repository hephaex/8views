fn main() {
    uniffi::generate_scaffolding("src/eightviews.udl").unwrap();
    println!("cargo:rerun-if-changed=src/eightviews.udl");
}
