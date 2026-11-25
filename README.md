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
| `sr25519` | sr25519 keys of Substrate wallets' recovery phrases (polkadot-js, SubWallet, Talisman, AirGap Vault), on `schnorrkel` and `substrate-bip39` |
| `processor-ffi` | The single native library for the apps: bindings of the P2P layer, sr25519, the SDK (not on Android) and the benchmark |

Each folder was imported from its original repository with `git subtree`, keeping its history.

## Build and test

```sh
cargo build
cargo test -- --skip acurast_rpc_tests   # these call a devnet node that no longer exists
```

## Packages for the apps

The apps get one native library, `acurast_processor_ffi` (crate `processor-ffi`), with the bindings
of the P2P layer, the sr25519 keys, the SDK (iOS only: Android uses the Kotlin SDK) and the
benchmark.

| platform | build | the app depends on |
|---|---|---|
| Android | `make android` (JDK 17, NDK 29.0.14206865, `android/local.properties` with `sdk.dir`); JitPack builds release tags with `jitpack.yml` | `com.github.Acurast:acurast-processor-libs:<tag>`: one AAR with the native library, the uniffi Kotlin bindings (`uniffi.acurast_p2p_ffi`, `uniffi.acurast_sr25519`), `com.acurast.p2p` and the benchmark (`com.acurast.bench`, JNI glue `libacubench.so` linked to the native library) |
| iOS | `make ios` (Xcode); the xcframework and the Swift bindings are committed | Swift package of this repo, products `AcurastP2P`, `AcurastSr25519`, `AcurastSDK`, `Acubench`. The C module of all of them is `AcurastProcessorFFI`, named after the xcframework because Xcode finds no other name |

Release builds use link-time optimization: without it the iOS simulator library is above GitHub's
100 MB file limit.

Run `make ios` and commit `ios/` whenever the Rust code of the libraries or their bindings change.

## Releasing a version

GitLab (`papers/acurast/acurast-processor-libs`) is the main repository. It is push-mirrored, branches
and tags, to the public `github.com/Acurast/acurast-processor-libs`, where JitPack and Swift Package
Manager read it.

1. With the Rust changes, run `make ios` and commit `ios/` in the same merge request, so the Swift
   package matches the code of the tag.
2. Merge into `main` once the pipeline is green.
3. Tag `main` with the version, without `v` (`0.1.3`), and push the tag to GitLab:
   `git tag -a 0.1.3 -m "0.1.3: <what changed>" && git push origin 0.1.3`.
   The mirror copies it to GitHub within a few minutes ("Update now" in GitLab's mirror settings
   forces it).
4. JitPack builds the AAR the first time the version is requested (about 10 minutes): open
   `https://jitpack.io/#Acurast/acurast-processor-libs/0.1.3` or request the POM, and check the build
   log there if it fails.
5. Update the apps: `acurastProcessorLibs` in the Android app's `gradle/libs.versions.toml`, and the
   exact version of the `acurast-processor-libs` package in the iOS app (commit its
   `Package.resolved`).

Never move or reuse a tag: Swift Package Manager remembers the commit of each version, and JitPack
keeps the first build. Fix a broken release with a new version.

