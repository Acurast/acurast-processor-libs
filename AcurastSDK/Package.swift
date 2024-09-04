// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: AcurastSDK

import PackageDescription;

let package = Package(
    name: "AcurastSDK",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "AcurastSDK",
            targets: ["AcurastSDK"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "AcurastSDKFFI", path: "./AcurastSDKFFI.xcframework"),
        .target(
            name: "AcurastSDK",
            dependencies: [
                .target(name: "AcurastSDKFFI")
            ]
        ),
    ]
)