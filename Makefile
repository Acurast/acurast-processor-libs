install-cargo-swift:
	cargo install cargo-swift@0.8.1 -f

build:
	cargo swift package --platforms ios --name AcurastSDK --xcframework-name AcurastSDKFFI --release --accept-all
