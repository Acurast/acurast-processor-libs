//! One native library for the apps instead of one per library: they share a single copy of Rust's
//! standard library and of the async runtime.
//!
//! The uniffi bindings of the P2P layer, of the sr25519 keys and of the SDK keep their own
//! namespaces (`acurast_p2p_ffi`, `acurast_sr25519`, `acurastsdk`); the benchmark keeps its C
//! functions (`bench_cpu`, ...).
//! The SDK is not part of the Android library: Android uses the Kotlin SDK.

acurast_p2p_ffi::uniffi_reexport_scaffolding!();
acurast_sr25519::uniffi_reexport_scaffolding!();
#[cfg(not(target_os = "android"))]
acurastsdk::uniffi_reexport_scaffolding!();

// Links the benchmark, whose C functions (Android and iOS only) are exported as they are.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use acubench as _;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use acubench::ffi as benchmark;
