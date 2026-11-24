// swift-tools-version:5.9
// The libraries of the Acurast processor for iOS, all in one native library (AcurastProcessorFFI,
// built with `make ios`).

import PackageDescription

// The native library is one object (link-time optimized), so every product links what any part of
// it needs: the P2P layer reads the network interfaces and DNS servers.
let systemFrameworks: [LinkerSetting] = [
    .linkedFramework("Network"),
    .linkedFramework("SystemConfiguration"),
]

let package = Package(
    name: "AcurastProcessorLibs",
    platforms: [
        .iOS("15.6"),
    ],
    products: [
        .library(name: "AcurastP2P", targets: ["AcurastP2P"]),
        .library(name: "AcurastSr25519", targets: ["AcurastSr25519"]),
        .library(name: "AcurastSDK", targets: ["AcurastSDK"]),
        .library(name: "Acubench", targets: ["Acubench"]),
    ],
    targets: [
        .binaryTarget(name: "AcurastProcessorFFI", path: "ios/AcurastProcessorFFI.xcframework"),
        .target(
            name: "AcurastP2P",
            dependencies: ["AcurastProcessorFFI"],
            path: "ios/Sources/AcurastP2P",
            linkerSettings: systemFrameworks
        ),
        .target(
            name: "AcurastSr25519",
            dependencies: ["AcurastProcessorFFI"],
            path: "ios/Sources/AcurastSr25519",
            linkerSettings: systemFrameworks
        ),
        .target(
            name: "AcurastSDK",
            dependencies: ["AcurastProcessorFFI"],
            path: "ios/Sources/AcurastSDK",
            linkerSettings: systemFrameworks
        ),
        .target(
            name: "Acubench",
            dependencies: ["AcurastProcessorFFI"],
            path: "ios/Sources/Acubench",
            resources: [.copy("Resources/PrivacyInfo.xcprivacy")],
            linkerSettings: systemFrameworks
        ),
        .testTarget(name: "AcubenchTests", dependencies: ["Acubench"], path: "ios/Tests/AcubenchTests"),
    ]
)
