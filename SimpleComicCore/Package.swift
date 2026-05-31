// swift-tools-version: 5.9
// SimpleComicCore — Swift Package wrapping the Rust libsimplecomic.a FFI layer
//
// Usage (in Xcode or Package.swift):
//   .package(path: "../SimpleComicCore")
//   .product(name: "SimpleComicCore", package: "SimpleComicCore")
//
// Before using, build the Rust universal library:
//   cd simple-comic-core
//   cargo build --release --package sc-ffi --target aarch64-apple-darwin
//   cargo build --release --package sc-ffi --target x86_64-apple-darwin
//   lipo -create \
//     target/aarch64-apple-darwin/release/libsimplecomic.a \
//     target/x86_64-apple-darwin/release/libsimplecomic.a \
//     -output ../SimpleComicCore/Libraries/libsimplecomic.a
//
// Regenerate Swift bindings after UDL changes:
//   cargo run --bin uniffi-bindgen -- generate \
//     sc-ffi/src/simplecomic.udl --language swift \
//     --out-dir ../SimpleComicCore/Sources/SimpleComicCore/

import PackageDescription

let package = Package(
    name: "SimpleComicCore",
    platforms: [.macOS(.v12)],
    products: [
        .library(name: "SimpleComicCore", targets: ["SimpleComicCore"]),
    ],
    targets: [
        // Swift wrapper (generated uniffi bindings + hand-written helpers)
        .target(
            name: "SimpleComicCore",
            dependencies: ["simplecomicFFI"],
            path: "Sources/SimpleComicCore"
        ),
        // C/Rust FFI layer — pre-built static library + generated header
        .target(
            name: "simplecomicFFI",
            path: "Sources/simplecomicFFI",
            publicHeadersPath: ".",
            linkerSettings: [
                .linkedLibrary("simplecomic", .when(platforms: [.macOS])),
                .unsafeFlags(["-L", "\(Context.packageDirectory)/Libraries"]),
                // Rust stdlib and system frameworks required by libsimplecomic.a
                .linkedLibrary("resolv"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
            ]
        ),
        .testTarget(
            name: "SimpleComicCoreTests",
            dependencies: ["SimpleComicCore"],
            path: "Tests/SimpleComicCoreTests"
        ),
    ]
)
