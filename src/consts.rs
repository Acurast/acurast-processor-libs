//! ALPNs, domain-separation tags, close codes and limits.

// ALPNs: one per service; the handler checks the caller's role.

/// Registry served by relay nodes.
pub const ALPN_REGISTRY: &[u8] = b"acurast/registry/1";
/// Network secret fetch served by processors.
pub const ALPN_SECRET_FETCH: &[u8] = b"acurast/secret-fetch/1";
/// Connections to jobs running on a processor.
pub const ALPN_JOB: &[u8] = b"acurast/job/1";
/// Notices of new job assignments, sent by relay nodes to processors.
pub const ALPN_ASSIGNMENTS: &[u8] = b"acurast/assignments/1";
/// Commands of a manager to its processors.
pub const ALPN_MANAGE: &[u8] = b"acurast/manage/1";

// Domain-separation tags: every signed or hashed value starts with one, so a signature or hash
// made for one purpose can never be reused for another.

/// `register_endpoint` proof of possession.
pub const TAG_REGISTER: &[u8] = b"acurast/p2p/register/v1";
/// Registry placement by rendezvous hashing.
pub const TAG_HRW: &[u8] = b"acurast/registry/hrw/v1";
/// Gossip topic ids.
pub const TAG_TOPIC: &[u8] = b"acurast/gossip/topic/v1";
/// Gossip message signatures.
pub const TAG_GOSSIP: &[u8] = b"acurast/gossip/msg/v1";
/// Network secret commitment.
pub const TAG_COMMIT: &[u8] = b"acurast/secret/commit/v1";

// Roles of an endpoint, as bits of the value returned by the `P2pApi_roles` runtime API.

/// The endpoint is registered by an account.
pub const ROLE_REGISTERED: u8 = 0b0001;
/// The endpoint is registered by an attested processor.
pub const ROLE_PROCESSOR: u8 = 0b0010;
/// The endpoint is registered by a manager.
pub const ROLE_MANAGER: u8 = 0b0100;
/// The endpoint is a relay node.
pub const ROLE_RELAY: u8 = 0b1000;

// Application close codes.

/// Normal close.
pub const CLOSE_NORMAL: u32 = 0;
/// The caller does not have the role required by the service.
pub const ERR_ROLE: u32 = 1;
/// A connection or rate limit was hit.
pub const ERR_LIMIT: u32 = 2;
/// The peer violated the protocol (bad frame, unexpected message).
pub const ERR_PROTOCOL: u32 = 3;

// Limits and parameters shared by clients and servers.

/// Maximum size of a framed message body in bytes.
pub const MAX_FRAME: usize = 64 * 1024;
/// Number of relays that hold the registry record of an endpoint.
pub const REGISTRY_REPLICAS: usize = 2;
/// Maximum lifetime of a registry record in seconds.
pub const REGISTRY_MAX_TTL_SECS: u32 = 600;
/// How often a client re-registers, in seconds.
pub const REGISTRY_REFRESH_SECS: u32 = 300;
/// Debounce before a client publishes a changed record, in seconds.
pub const PUBLISH_DEBOUNCE_SECS: u32 = 2;
/// Maximum number of direct addresses in a registry record.
pub const MAX_DIRECT_ADDRS: usize = 8;
/// Maximum number of namespaces in a registry record.
pub const MAX_NAMESPACES: usize = 8;
/// Maximum number of records returned by one `Discover`.
pub const MAX_DISCOVER_LIMIT: u16 = 100;
/// Maximum length of a relay URL in bytes, on chain and in registry records.
pub const MAX_RELAY_URL_LEN: usize = 128;
/// Maximum length of a namespace in bytes.
pub const MAX_NAMESPACE_LEN: usize = 64;
/// Maximum length of a job-defined protocol name in `JobHello`.
pub const MAX_JOB_PROTOCOL_LEN: usize = 64;
/// Jobs listed in one assignment notice.
pub const MAX_NOTICE_JOBS: usize = 64;
/// Maximum age of a manager command in seconds: older commands are refused.
pub const MANAGE_MAX_AGE_SECS: u64 = 60;
/// Maximum length of a text field of a processor status in bytes.
pub const MAX_STATUS_TEXT_LEN: usize = 64;
/// Maximum gossip payload in bytes.
pub const GOSSIP_MAX_PAYLOAD: usize = 4 * 1024;
