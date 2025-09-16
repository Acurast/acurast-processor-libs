install-cargo-swift:
	cargo install cargo-swift@0.9.0 -f

build:
	cargo swift package --platforms ios --name AcurastSDK --xcframework-name AcurastSDKFFI --release --accept-all
