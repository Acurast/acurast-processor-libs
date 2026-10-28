//! Shared protocol definitions of the Acurast P2P layer (iroh).
//!
//! Used by the node (relay, registry), `pallet-acurast-p2p` and the processor library, so that
//! all of them agree on service names (ALPNs), domain-separation tags, hashes, message types
//! and framing.
//!
//! Without the `std` feature only [`consts`], [`hash`] and [`namespace`] are available: that is
//! what the runtime needs (e.g. [`hash::register_proof_message`]).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod consts;
pub mod hash;
pub mod namespace;

#[cfg(feature = "std")]
pub mod frame;
#[cfg(feature = "std")]
pub mod messages;

/// iroh endpoint id: an Ed25519 public key.
pub type EndpointId = [u8; 32];

pub use consts::*;
