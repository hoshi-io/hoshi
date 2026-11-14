// swift-tools-version:5.5

import PackageDescription

let package = Package(
    name: "tauri-plugin-hoshi-ios",
    platforms: [
        .macOS(.v10_13),
        .iOS(.v15),
    ],
    products: [
        .library(
            name: "tauri-plugin-hoshi-ios",
            type: .static,
            targets: ["tauri-plugin-hoshi-ios"])
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .systemLibrary(name: "HoshiSysLibs", path: "SysLibs"),
        .target(
            name: "tauri-plugin-hoshi-ios",
            dependencies: [
                .byName(name: "Tauri"),
                .byName(name: "HoshiSysLibs"),
            ],
            path: "Sources")
    ]
)
