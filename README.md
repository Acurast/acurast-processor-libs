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

Each folder was imported from its original repository with `git subtree`, keeping its history.
