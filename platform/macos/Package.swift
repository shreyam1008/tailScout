// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TailScout",
    defaultLocalization: "en",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "TailScout", targets: ["TailScout"])
    ],
    targets: [
        .target(
            name: "TailScoutCore",
            path: "Sources/TailScoutCore"
        ),
        .executableTarget(
            name: "TailScout",
            dependencies: ["TailScoutCore"],
            path: "Sources/TailScout"
        ),
        .testTarget(
            name: "TailScoutTests",
            dependencies: ["TailScoutCore"],
            path: "Tests/TailScoutTests"
        )
    ]
)
