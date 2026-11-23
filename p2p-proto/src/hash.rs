//! Hashes and signed payloads shared by chain, relays and processors. All hashes are `blake2_256`
//! (BLAKE2b with 32-byte output), the same function as Substrate's `blake2_256`.

use alloc::vec::Vec;

use blake2::{digest::consts::U32, Blake2b, Digest};

use crate::{
	consts::{TAG_COMMIT, TAG_HRW, TAG_REGISTER, TAG_TOPIC},
	EndpointId,
};

/// `blake2_256` over the concatenation of `parts`.
pub fn blake2_256(parts: &[&[u8]]) -> [u8; 32] {
	let mut hasher = Blake2b::<U32>::new();
	for part in parts {
		hasher.update(part);
	}
	hasher.finalize().into()
}

/// Message the iroh key signs to prove possession in `register_endpoint`:
/// `TAG_REGISTER ‖ genesis_hash ‖ SCALE(who)`.
///
/// `who_scale` is the SCALE encoding of the registering account (for `AccountId32`: its 32
/// bytes). The genesis hash prevents replay on another network; `who` binds the key to the
/// account.
pub fn register_proof_message(genesis_hash: &[u8; 32], who_scale: &[u8]) -> Vec<u8> {
	[TAG_REGISTER, genesis_hash.as_slice(), who_scale].concat()
}

/// Gossip topic id: `blake2_256(TAG_TOPIC ‖ name)`.
pub fn topic_id(name: &str) -> [u8; 32] {
	blake2_256(&[TAG_TOPIC, name.as_bytes()])
}

/// Network secret commitment: `blake2_256(TAG_COMMIT ‖ G ‖ S)`, with the
/// generation `G` SCALE-encoded (`u32` little-endian), as for every value hashed on chain.
pub fn secret_commitment(generation: u32, secret: &[u8; 32]) -> [u8; 32] {
	blake2_256(&[TAG_COMMIT, &generation.to_le_bytes(), secret])
}

/// Rendezvous score of `relay` for `endpoint`: `blake2_256(TAG_HRW ‖ endpoint ‖ relay)`.
/// Higher scores win, comparing the hashes as big-endian numbers.
pub fn hrw_score(endpoint: &EndpointId, relay: &EndpointId) -> [u8; 32] {
	blake2_256(&[TAG_HRW, endpoint, relay])
}

/// The `replicas` relays that hold the registry record of `endpoint`: those with the highest
/// [`hrw_score`], best first. Every client computes the same result from the same relay set,
/// and a change of the relay set only moves the records of the affected relays.
pub fn home_registries(
	endpoint: &EndpointId,
	relays: &[EndpointId],
	replicas: usize,
) -> Vec<EndpointId> {
	let mut scored: Vec<([u8; 32], EndpointId)> =
		relays.iter().map(|relay| (hrw_score(endpoint, relay), *relay)).collect();
	scored.sort_unstable_by(|a, b| b.cmp(a));
	scored.dedup_by(|a, b| a.1 == b.1);
	scored.into_iter().take(replicas).map(|(_, relay)| relay).collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	#[test]
	fn blake2_256_matches_substrate() {
		// Substrate's blake2_256(b"abc").
		assert_eq!(
			blake2_256(&[b"abc"]),
			hex!("bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319")
		);
		assert_eq!(blake2_256(&[b"a", b"bc"]), blake2_256(&[b"abc"]));
	}

	#[test]
	fn register_proof_message_layout() {
		let msg = register_proof_message(&[1; 32], &[2; 32]);
		assert_eq!(msg.len(), TAG_REGISTER.len() + 64);
		assert!(msg.starts_with(TAG_REGISTER));
		assert_eq!(&msg[TAG_REGISTER.len()..][..32], &[1; 32]);
		assert_eq!(&msg[TAG_REGISTER.len() + 32..], &[2; 32]);
	}

	#[test]
	fn secret_commitment_depends_on_generation() {
		assert_ne!(secret_commitment(1, &[7; 32]), secret_commitment(2, &[7; 32]));
	}

	fn relays(n: u8) -> Vec<EndpointId> {
		(0..n).map(|i| [i; 32]).collect()
	}

	#[test]
	fn home_registries_is_deterministic_and_order_independent() {
		let endpoint = [42; 32];
		let set = relays(10);
		let mut reversed = set.clone();
		reversed.reverse();

		let homes = home_registries(&endpoint, &set, 2);
		assert_eq!(homes.len(), 2);
		assert_ne!(homes[0], homes[1]);
		assert_eq!(homes, home_registries(&endpoint, &reversed, 2));
	}

	#[test]
	fn home_registries_is_stable_when_an_unrelated_relay_leaves() {
		let endpoint = [42; 32];
		let set = relays(10);
		let homes = home_registries(&endpoint, &set, 2);

		// Removing a relay that is not a home of the endpoint must not change its homes.
		let other = *set.iter().find(|r| !homes.contains(r)).unwrap();
		let smaller: Vec<_> = set.iter().copied().filter(|r| *r != other).collect();
		assert_eq!(homes, home_registries(&endpoint, &smaller, 2));
	}

	#[test]
	fn home_registries_with_few_relays() {
		assert_eq!(home_registries(&[1; 32], &relays(1), 2).len(), 1);
		assert!(home_registries(&[1; 32], &[], 2).is_empty());
		// Duplicates in the relay set are ignored.
		assert_eq!(home_registries(&[1; 32], &[[5; 32], [5; 32]], 2), vec![[5; 32]]);
	}
}
