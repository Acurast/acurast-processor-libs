# acurast-processor-libs

Rust libraries of the Acurast processors (Android and iOS), in one workspace so the apps can ship them
as a single native library.

| folder | what |
|---|---|
| `p2p-proto` | Protocol definitions of the P2P layer, shared with the chain and the relay nodes |
| `p2p-registry` | Registry of the P2P layer, served by relay nodes |
| `p2p` | P2P layer of the processors (library, FFI bindings, CLI for managers) |
| `rust-sdk` | Acurast SDK (light client) |
| `benchmark` | Device benchmark |
| `processor-ffi` | The single native library for the apps: bindings of the P2P layer, the SDK (not on Android) and the benchmark |

Each folder was imported from its original repository with `git subtree`, keeping its history.

## Build and test

```sh
cargo build
cargo test -- --skip acurast_rpc_tests   # these call a devnet node that no longer exists
```

## Packages for the apps

The apps get one native library, `acurast_processor_ffi` (crate `processor-ffi`), with the bindings
of the P2P layer, the SDK (iOS only: Android uses the Kotlin SDK) and the benchmark.

| platform | build | the app depends on |
|---|---|---|
| Android | `make android` (JDK 17, NDK 29.0.14206865, `android/local.properties` with `sdk.dir`); JitPack builds release tags with `jitpack.yml` | `com.github.acurast:acurast-processor-libs:<tag>`: one AAR with the native library, the uniffi Kotlin bindings (`uniffi.acurast_p2p_ffi`), `com.acurast.p2p` and the benchmark (`com.acurast.bench`, JNI glue `libacubench.so` linked to the native library) |
| iOS | `make ios` (Xcode); the xcframework and the Swift bindings are committed | Swift package of this repo, products `AcurastP2P`, `AcurastSDK`, `Acubench`. The C module of all of them is `AcurastProcessorFFI`, named after the xcframework because Xcode finds no other name |

Release builds use link-time optimization: without it the iOS simulator library is above GitHub's
100 MB file limit.

Run `make ios` and commit `ios/` whenever the Rust code of the libraries or their bindings change.

