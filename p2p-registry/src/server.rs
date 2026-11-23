//! The registry as a protocol of an iroh endpoint: one request and one response per bi stream.

use std::{
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};

use acurast_p2p_proto::{
	frame::{self, HEADER_LEN},
	messages::{Record, RegistryRequest, RegistryResponse},
	EndpointId, ERR_LIMIT, ERR_PROTOCOL, ERR_ROLE, MAX_FRAME, ROLE_REGISTERED, ROLE_RELAY,
};
use iroh::{
	endpoint::{Connection, RecvStream, SendStream, VarInt},
	protocol::{AcceptError, ProtocolHandler},
};

use crate::store::Store;

const LOG_TARGET: &str = "p2p-registry";

/// Requests a connection may send per second before it is closed.
const MAX_REQUESTS_PER_SEC: u32 = 10;

/// Roles of the endpoints that connect to the registry.
pub trait RoleSource: Send + Sync + 'static {
	/// Role bits of the endpoint `id` (`acurast_p2p_proto::ROLE_*`).
	fn roles(&self, id: &EndpointId) -> Result<u8, String>;
}

/// Serves the registry to endpoints that are registered on chain or are relay nodes; other
/// connections are closed with `ERR_ROLE`. Roles are read once per connection.
#[derive(Clone)]
pub struct Registry {
	store: Arc<Mutex<Store>>,
	roles: Arc<dyn RoleSource>,
}

impl Registry {
	pub fn new(roles: Arc<dyn RoleSource>) -> Self {
		Self::with_store(Store::default(), roles)
	}

	pub fn with_store(store: Store, roles: Arc<dyn RoleSource>) -> Self {
		Self { store: Arc::new(Mutex::new(store)), roles }
	}

	/// Number of records held.
	pub fn len(&self) -> usize {
		self.store.lock().expect("registry store lock poisoned").len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Record of `id` held here, if present and not expired.
	pub fn lookup(&self, id: &EndpointId) -> Option<Record> {
		let store = self.store.lock().expect("registry store lock poisoned");
		store.lookup(id, Instant::now())
	}

	fn handle(&self, caller: EndpointId, roles: u8, request: RegistryRequest) -> RegistryResponse {
		let mut store = self.store.lock().expect("registry store lock poisoned");
		store.handle(caller, roles, request, Instant::now())
	}

	/// Answers one request; `Err` with a close code when the caller broke the protocol.
	async fn serve_stream(
		&self,
		caller: EndpointId,
		roles: u8,
		mut send: SendStream,
		mut recv: RecvStream,
	) -> Result<(), u32> {
		let bytes = recv.read_to_end(HEADER_LEN + MAX_FRAME).await.map_err(|_| ERR_PROTOCOL)?;
		let request = match frame::decode::<RegistryRequest>(&bytes) {
			Ok((request, used)) if used == bytes.len() => request,
			_ => return Err(ERR_PROTOCOL),
		};
		let summary = summary(&request);
		let response = self.handle(caller, roles, request);
		log::debug!(target: LOG_TARGET, "{}: {summary} -> {}", hex(&caller), outcome(&response));
		let frame = frame::encode(&response).map_err(|_| ERR_PROTOCOL)?;
		// The caller may go away meanwhile; that only ends its stream.
		if send.write_all(&frame).await.is_ok() {
			let _ = send.finish();
		}
		Ok(())
	}
}

impl std::fmt::Debug for Registry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Registry").field("records", &self.len()).finish()
	}
}

impl ProtocolHandler for Registry {
	async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
		let caller = *connection.remote_id().as_bytes();
		let roles = match self.roles.roles(&caller) {
			Ok(roles) if roles & (ROLE_REGISTERED | ROLE_RELAY) != 0 => roles,
			Ok(_) => {
				close(&connection, ERR_ROLE, "not registered");
				return Ok(());
			},
			Err(e) => {
				log::debug!(target: LOG_TARGET, "Cannot read the roles of {}: {e}", connection.remote_id());
				close(&connection, ERR_ROLE, "not registered");
				return Ok(());
			},
		};

		let mut limiter = RateLimiter::new(MAX_REQUESTS_PER_SEC);
		// Ends when the caller closes the connection.
		while let Ok((send, recv)) = connection.accept_bi().await {
			if !limiter.allow(Instant::now()) {
				close(&connection, ERR_LIMIT, "too many requests");
				return Ok(());
			}
			if let Err(code) = self.serve_stream(caller, roles, send, recv).await {
				close(&connection, code, "invalid request");
				return Ok(());
			}
		}
		Ok(())
	}
}

/// The request without the addresses, for the debug log.
fn summary(request: &RegistryRequest) -> String {
	match request {
		RegistryRequest::Register { seq, namespaces, .. } => {
			format!("Register(seq {seq}, namespaces {namespaces:?})")
		},
		RegistryRequest::Unregister => "Unregister".into(),
		RegistryRequest::Lookup { id } => format!("Lookup({})", hex(id)),
		RegistryRequest::Discover { namespace, .. } => format!("Discover({namespace})"),
	}
}

/// The response without the records' addresses, for the debug log.
fn outcome(response: &RegistryResponse) -> String {
	match response {
		RegistryResponse::Registered { ttl_secs } => format!("Registered(ttl {ttl_secs}s)"),
		RegistryResponse::Ok => "Ok".into(),
		RegistryResponse::Found(Some(record)) => format!("Found(seq {})", record.seq),
		RegistryResponse::Found(None) => "NotFound".into(),
		RegistryResponse::Discovered { records, cursor } => {
			format!("Discovered({} records, more: {})", records.len(), cursor.is_some())
		},
		RegistryResponse::Err(e) => format!("{e:?}"),
	}
}

fn hex(id: &EndpointId) -> String {
	id.iter().map(|b| format!("{b:02x}")).collect()
}

fn close(connection: &Connection, code: u32, reason: &str) {
	connection.close(VarInt::from_u32(code), reason.as_bytes());
}

/// Allows at most `limit` events per one-second window.
struct RateLimiter {
	limit: u32,
	window_start: Option<Instant>,
	count: u32,
}

impl RateLimiter {
	fn new(limit: u32) -> Self {
		Self { limit, window_start: None, count: 0 }
	}

	fn allow(&mut self, now: Instant) -> bool {
		match self.window_start {
			Some(start) if now.duration_since(start) < Duration::from_secs(1) => {},
			_ => {
				self.window_start = Some(now);
				self.count = 0;
			},
		}
		self.count += 1;
		self.count <= self.limit
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rate_limiter_allows_the_limit_per_second() {
		let mut limiter = RateLimiter::new(2);
		let start = Instant::now();

		assert!(limiter.allow(start));
		assert!(limiter.allow(start));
		assert!(!limiter.allow(start + Duration::from_millis(500)));
		assert!(limiter.allow(start + Duration::from_secs(1)));
	}
}
