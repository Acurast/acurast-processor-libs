//! Message types exchanged over the P2P services, encoded with postcard.
//!
//! Enum variants are encoded by index: new variants may only be appended at the end.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{consts::TAG_GOSSIP, EndpointId};

// Registry, ALPN `acurast/registry/1`.

/// What the registry stores about an endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
	/// Always the authenticated `conn.remote_id()` of the publisher, never taken from a request.
	pub id: EndpointId,
	/// Publisher-chosen, monotonic (unix ms).
	pub seq: u64,
	/// Home relay URL.
	pub relay_url: Option<String>,
	/// At most [`crate::MAX_DIRECT_ADDRS`].
	pub direct: Vec<SocketAddr>,
	/// At most [`crate::MAX_NAMESPACES`].
	pub namespaces: Vec<String>,
	/// Set by the server in responses.
	pub expires_in_secs: u32,
}

/// Requests to the registry; one per bi stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryRequest {
	/// Replaces the caller's record. `ttl_secs` is capped at [`crate::REGISTRY_MAX_TTL_SECS`].
	Register {
		seq: u64,
		relay_url: Option<String>,
		direct: Vec<SocketAddr>,
		namespaces: Vec<String>,
		ttl_secs: u32,
	},
	/// Removes the caller's record.
	Unregister,
	/// Record of `id` held by this server.
	Lookup { id: EndpointId },
	/// Members of `namespace` held by this server, in pages of at most `limit`
	/// ([`crate::MAX_DISCOVER_LIMIT`]). `cursor` is `None` on the first call and then the value
	/// returned by the previous page.
	Discover { namespace: String, cursor: Option<u32>, limit: u16 },
}

/// Responses of the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryResponse {
	/// Answer to `Register`, with the TTL the server applied.
	Registered {
		ttl_secs: u32,
	},
	/// Answer to `Unregister`.
	Ok,
	/// Answer to `Lookup`: the record, if present and not expired.
	Found(Option<Record>),
	/// Answer to `Discover`; `cursor` is `None` when there are no more pages.
	Discovered {
		records: Vec<Record>,
		cursor: Option<u32>,
	},
	Err(RegistryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryError {
	/// The namespace is not known to the server.
	UnknownNamespace,
	/// The caller's role does not allow this request (e.g. registering in a namespace).
	Forbidden,
	/// `Register` with a `seq` not greater than the stored one.
	StaleSeq,
	/// The server is at its record or request limit.
	Busy,
	/// The request breaks a limit (too many addresses, invalid namespace, ...).
	Invalid,
}

// Jobs, ALPN `acurast/job/1`.

/// First message on a connection to a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobHello {
	/// SCALE-encoded `JobId<AccountId32>` = `(MultiOrigin, u128)`.
	pub target_job: Vec<u8>,
	/// Job-defined, at most [`crate::MAX_JOB_PROTOCOL_LEN`] bytes.
	pub protocol: String,
	/// Filled by the dialing processor, never by the job.
	pub source_job: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobHelloReply {
	Ok,
	Err(JobError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobError {
	NotRunningHere,
	NoListener,
	Busy,
	Forbidden,
}

// Network secret, ALPN `acurast/secret-fetch/1`.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretRequest {
	pub generation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretResponse {
	Secret { generation: u32, secret: [u8; 32] },
	Err(SecretError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretError {
	NotHeld,
	Forbidden,
	Busy,
}

// Gossip.

/// Gossip payload signed by its author; the gossip layer itself only authenticates the hop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedGossip {
	pub author: EndpointId,
	/// Per author, monotonic.
	pub seq: u64,
	pub sent_at_ms: u64,
	pub payload: Vec<u8>,
	/// Ed25519 signature by `author` over [`SignedGossip::signing_payload`].
	#[serde(with = "signature_bytes")]
	pub sig: [u8; 64],
}

impl SignedGossip {
	/// What the author signs: `TAG_GOSSIP ‖ topic_id ‖ postcard(author, seq, sent_at_ms, payload)`.
	pub fn signing_payload(
		topic_id: &[u8; 32],
		author: &EndpointId,
		seq: u64,
		sent_at_ms: u64,
		payload: &[u8],
	) -> Vec<u8> {
		let body = postcard::to_stdvec(&(author, seq, sent_at_ms, payload))
			.expect("serializing plain data into a Vec cannot fail");
		[TAG_GOSSIP, topic_id.as_slice(), &body].concat()
	}
}

/// serde supports arrays only up to 32 elements; signatures are 64 bytes.
mod signature_bytes {
	use serde::{de::Error, Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer>(sig: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
		s.serialize_bytes(sig)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
		let bytes = <&[u8]>::deserialize(d)?;
		bytes.try_into().map_err(|_| D::Error::invalid_length(bytes.len(), &"64 bytes"))
	}
}
