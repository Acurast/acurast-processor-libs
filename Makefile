# Packages for the apps, from the single native library (processor-ffi).
#   make ios      Swift package: ios/AcurastProcessorFFI.xcframework + Swift bindings (committed)
#   make android  Android library (AAR) in android/lib/build/outputs/aar

IOS_TARGETS := aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
# Same minimum as the processor app; the benchmark's C++ code needs it.
export IPHONEOS_DEPLOYMENT_TARGET := 15.6

LIB := libacurast_processor_ffi.a
FFI := AcurastProcessorFFI
XCFRAMEWORK := ios/$(FFI).xcframework
BUILD := target/ios-package

ios:
	rustup target add $(IOS_TARGETS)
	$(foreach t,$(IOS_TARGETS),cargo build --release -p acurast-processor-ffi --target $(t) &&) true
	rm -rf $(BUILD) && mkdir -p $(BUILD)/sim $(BUILD)/headers/$(FFI) $(BUILD)/swift
	lipo -create target/aarch64-apple-ios-sim/release/$(LIB) target/x86_64-apple-ios/release/$(LIB) -output $(BUILD)/sim/$(LIB)
	# Swift bindings and C headers of the uniffi libraries, read from the library itself.
	cargo run --release -p acurast-processor-ffi --features uniffi/cli --bin uniffi-bindgen -- generate \
		--library target/aarch64-apple-ios/release/$(LIB) --language swift --out-dir $(BUILD)/swift
	cp $(BUILD)/swift/*.h benchmark/rust/ffi/include/ffi.h $(BUILD)/headers/$(FFI)/
	# One C module, named after the xcframework (the only name Xcode finds), with every header.
	printf 'module $(FFI) {\n    header "acurast_p2p_ffiFFI.h"\n    header "acurastsdkFFI.h"\n    header "ffi.h"\n    export *\n}\n' > $(BUILD)/headers/$(FFI)/module.modulemap
	rm -rf $(XCFRAMEWORK)
	xcodebuild -create-xcframework \
		-library target/aarch64-apple-ios/release/$(LIB) -headers $(BUILD)/headers \
		-library $(BUILD)/sim/$(LIB) -headers $(BUILD)/headers \
		-output $(XCFRAMEWORK)
	mkdir -p ios/Sources/AcurastP2P ios/Sources/AcurastSDK
	cp $(BUILD)/swift/acurast_p2p_ffi.swift ios/Sources/AcurastP2P/
	cp $(BUILD)/swift/acurastsdk.swift ios/Sources/AcurastSDK/

android:
	cd android && ./gradlew :lib:assembleRelease

.PHONY: ios android
