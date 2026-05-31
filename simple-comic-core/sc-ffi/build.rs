fn main() {
    // Enable uniffi scaffolding generation once the UDL is finalised.
    // Sprint 6: uncomment the line below to generate Swift/Kotlin bindings from
    // src/simplecomic.udl, then add `uniffi::include_scaffolding!("simplecomic")`
    // to lib.rs.
    //
    // uniffi::generate_scaffolding("src/simplecomic.udl").unwrap();
    //
    // Re-run the build script whenever the UDL definition changes so that
    // incremental builds stay consistent.
    println!("cargo:rerun-if-changed=src/simplecomic.udl");
}
