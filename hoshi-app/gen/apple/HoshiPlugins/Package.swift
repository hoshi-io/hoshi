// swift-tools-version:5.5
// Built by hoshi-app/build.rs (swift-rs) into the Rust staticlib, not by the Xcode project:
// the plugins need the Tauri module, which only exists on the Rust side.

import PackageDescription

let package = Package(
    name: "hoshi-plugins",
    platforms: [
        .macOS(.v10_13),
        .iOS(.v15),
    ],
    products: [
        .library(
            name: "hoshi-plugins",
            type: .static,
            targets: ["hoshi-plugins"])
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .systemLibrary(name: "HoshiSysLibs", path: "SysLibs"),
        .target(
            name: "hoshi-plugins",
            dependencies: [
                .byName(name: "Tauri"),
                .byName(name: "HoshiSysLibs"),
            ],
            path: "Sources")
    ]
)
