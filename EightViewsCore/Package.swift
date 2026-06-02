// swift-tools-version: 5.9
// EightViewsCore — Swift Package wrapping the Rust libeightviews.a FFI layer
//
// Usage (in Xcode or Package.swift):
//   .package(path: "../EightViewsCore")
//   .product(name: "EightViewsCore", package: "EightViewsCore")
//
// Before using, build the Rust universal library:
//   cd 8views-core
//   cargo build --release --package sc-ffi --target aarch64-apple-darwin
//   cargo build --release --package sc-ffi --target x86_64-apple-darwin
//   lipo -create \
//     target/aarch64-apple-darwin/release/libeightviews.a \
//     target/x86_64-apple-darwin/release/libeightviews.a \
//     -output ../EightViewsCore/Libraries/libeightviews.a
//
// Regenerate Swift bindings after UDL changes:
//   cargo run --bin uniffi-bindgen -- generate \
//     sc-ffi/src/eightviews.udl --language swift \
//     --out-dir ../EightViewsCore/Sources/EightViewsCore/

import PackageDescription

let package = Package(
    name: "EightViewsCore",
    platforms: [.macOS(.v12)],
    products: [
        .library(name: "EightViewsCore", targets: ["EightViewsCore"]),
    ],
    targets: [
        // Swift wrapper (generated uniffi bindings + hand-written helpers)
        .target(
            name: "EightViewsCore",
            dependencies: ["eightviewsFFI"],
            path: "Sources/EightViewsCore"
        ),
        // C/Rust FFI layer — pre-built static library + generated header
        .target(
            name: "eightviewsFFI",
            path: "Sources/eightviewsFFI",
            publicHeadersPath: ".",
            linkerSettings: [
                .linkedLibrary("eightviews", .when(platforms: [.macOS])),
                .unsafeFlags(["-L", "\(Context.packageDirectory)/Libraries"]),
                // Transitive C/system dependencies of libeightviews.a
                .linkedLibrary("bz2"),       // bzip2 — TAR.BZ2 support (macOS SDK)
                .linkedLibrary("lzma"),      // XZ — TAR.XZ support (Homebrew)
                .linkedLibrary("c++"),       // C++ stdlib — unrar-ng (bundled C++ source)
                .unsafeFlags(["-L/opt/homebrew/lib"]),  // Homebrew lib path for liblzma
                // Rust stdlib and system frameworks required by libeightviews.a
                .linkedLibrary("resolv"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
            ]
        ),
        .testTarget(
            name: "EightViewsCoreTests",
            dependencies: ["EightViewsCore"],
            path: "Tests/EightViewsCoreTests"
        ),
    ]
)
