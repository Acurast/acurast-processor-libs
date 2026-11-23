IOS_RUST_TARGETS := aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

install-cargo-swift:
	cargo install cargo-swift@0.9.0 -f

ensure-ios-targets:
	rustup target add $(IOS_RUST_TARGETS)

build: ensure-ios-targets
	cargo swift package --platforms ios --name AcurastSDK --xcframework-name AcurastSDKFFI --release --accept-all
