//! Registry of the Acurast P2P layer, served by every relay node on ALPN `acurast/registry/1`.
//!
//! Endpoints publish where they can be reached (home relay, direct addresses) and the namespaces
//! they joined; others look them up by id or list the members of a namespace. Records live in
//! memory only and expire unless refreshed.

mod server;
pub mod store;

pub use server::{Registry, RoleSource};
