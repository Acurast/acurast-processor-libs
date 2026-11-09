//! In-memory registry records and the rules of the registry requests.

use std::{
	collections::{hash_map::RandomState, HashMap},
	hash::{BuildHasher, Hasher},
	time::{Duration, Instant},
};

use acurast_p2p_proto::{
	messages::{Record, RegistryError, RegistryRequest, RegistryResponse},
	namespace, EndpointId, MAX_DIRECT_ADDRS, MAX_DISCOVER_LIMIT, MAX_NAMESPACES, MAX_RELAY_URL_LEN,
	REGISTRY_MAX_TTL_SECS, ROLE_PROCESSOR,
};

/// Upper bound of the stored records; `Register` of a new endpoint fails with `Busy` beyond it.
pub const MAX_RECORDS: usize = 50_000;

struct Stored {
	record: Record,
	expires_at: Instant,
}

/// The records held by this relay node.
pub struct Store {
	records: HashMap<EndpointId, Stored>,
	max_records: usize,
}

impl Default for Store {
	fn default() -> Self {
		Self::with_capacity(MAX_RECORDS)
	}
}

impl Store {
	pub fn with_capacity(max_records: usize) -> Self {
		Self { records: HashMap::new(), max_records }
	}

	/// Answers `request` from `caller`, whose role bits are `roles`, at time `now`.
	pub fn handle(
		&mut self,
		caller: EndpointId,
		roles: u8,
		request: RegistryRequest,
		now: Instant,
	) -> RegistryResponse {
		self.prune(now);
		let result = match request {
			RegistryRequest::Register { seq, relay_url, direct, namespaces, ttl_secs } => self
				.register(
					caller,
					roles,
					Record { id: caller, seq, relay_url, direct, namespaces, expires_in_secs: 0 },
					ttl_secs,
					now,
				),
			RegistryRequest::Unregister => {
				self.records.remove(&caller);
				Ok(RegistryResponse::Ok)
			},
			RegistryRequest::Lookup { id } => Ok(RegistryResponse::Found(self.lookup(&id, now))),
			RegistryRequest::Discover { namespace, cursor, limit } => {
				self.discover(&namespace, cursor, limit, now)
			},
		};
		result.unwrap_or_else(RegistryResponse::Err)
	}

	/// Number of records held, expired ones included until the next request.
	pub fn len(&self) -> usize {
		self.records.len()
	}

	pub fn is_empty(&self) -> bool {
		self.records.is_empty()
	}

	fn register(
		&mut self,
		caller: EndpointId,
		roles: u8,
		record: Record,
		ttl_secs: u32,
		now: Instant,
	) -> Result<RegistryResponse, RegistryError> {
		if ttl_secs == 0
			|| record.direct.len() > MAX_DIRECT_ADDRS
			|| record.namespaces.len() > MAX_NAMESPACES
			|| record.relay_url.as_ref().is_some_and(|url| url.len() > MAX_RELAY_URL_LEN)
		{
			return Err(RegistryError::Invalid);
		}
		for ns in &record.namespaces {
			let required = required_role(ns)?;
			if roles & required != required {
				return Err(RegistryError::Forbidden);
			}
		}
		match self.records.get(&caller) {
			Some(stored) if record.seq <= stored.record.seq => return Err(RegistryError::StaleSeq),
			None if self.records.len() >= self.max_records => return Err(RegistryError::Busy),
			_ => {},
		}

		let ttl_secs = ttl_secs.min(REGISTRY_MAX_TTL_SECS);
		let expires_at = now + Duration::from_secs(ttl_secs.into());
		self.records.insert(caller, Stored { record, expires_at });
		Ok(RegistryResponse::Registered { ttl_secs })
	}

	/// Record of `id` at time `now`, if present and not expired.
	pub fn lookup(&self, id: &EndpointId, now: Instant) -> Option<Record> {
		self.records.get(id).map(|stored| answer(stored, now))
	}

	/// Pages through the members of `ns` sorted by id, starting at a random member so that
	/// clients asking for one page get different samples. The cursor packs the start (high 16
	/// bits) and the number of members already returned (low 16 bits), which both stay below
	/// [`MAX_RECORDS`].
	fn discover(
		&self,
		ns: &str,
		cursor: Option<u32>,
		limit: u16,
		now: Instant,
	) -> Result<RegistryResponse, RegistryError> {
		required_role(ns)?;
		if limit == 0 || limit > MAX_DISCOVER_LIMIT {
			return Err(RegistryError::Invalid);
		}

		let mut members: Vec<&Stored> = self
			.records
			.values()
			.filter(|stored| stored.record.namespaces.iter().any(|n| n == ns))
			.collect();
		members.sort_unstable_by_key(|stored| stored.record.id);
		let count = members.len();
		if count == 0 {
			return Ok(RegistryResponse::Discovered { records: Vec::new(), cursor: None });
		}

		let (start, taken) = match cursor {
			Some(cursor) => ((cursor >> 16) as usize % count, (cursor & 0xffff) as usize),
			None => (random_index(count), 0),
		};
		let page = usize::from(limit).min(count.saturating_sub(taken));
		let records = (taken..taken + page)
			.map(|i| answer(members[(start + i) % count], now))
			.collect();
		let taken = taken + page;
		let cursor = (taken < count).then_some(((start as u32) << 16) | taken as u32);
		Ok(RegistryResponse::Discovered { records, cursor })
	}

	fn prune(&mut self, now: Instant) {
		self.records.retain(|_, stored| stored.expires_at > now);
	}
}

/// The role needed to register into `ns`, or `UnknownNamespace` for a namespace this server does
/// not serve.
fn required_role(ns: &str) -> Result<u8, RegistryError> {
	if !namespace::is_valid(ns) {
		return Err(RegistryError::Invalid);
	}
	match ns.strip_prefix("secret-holder/").map(str::parse::<u32>) {
		Some(Ok(_)) => Ok(ROLE_PROCESSOR),
		_ => Err(RegistryError::UnknownNamespace),
	}
}

fn answer(stored: &Stored, now: Instant) -> Record {
	let remaining = stored.expires_at.saturating_duration_since(now).as_secs();
	Record { expires_in_secs: remaining.try_into().unwrap_or(u32::MAX), ..stored.record.clone() }
}

/// A random index below `count`, from the standard library's per-process random hash keys.
fn random_index(count: usize) -> usize {
	let mut hasher = RandomState::new().build_hasher();
	hasher.write_usize(count);
	(hasher.finish() % count as u64) as usize
}

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use super::*;

	use acurast_p2p_proto::ROLE_REGISTERED;

	const REGISTERED: u8 = ROLE_REGISTERED;
	const PROCESSOR: u8 = ROLE_REGISTERED | ROLE_PROCESSOR;
	const HOLDERS: &str = "secret-holder/1";

	fn register(seq: u64, namespaces: &[&str]) -> RegistryRequest {
		RegistryRequest::Register {
			seq,
			relay_url: Some("https://relay.example.com".into()),
			direct: vec!["203.0.113.7:7843".parse().unwrap()],
			namespaces: namespaces.iter().map(|ns| ns.to_string()).collect(),
			ttl_secs: 300,
		}
	}

	fn lookup(store: &mut Store, id: EndpointId, now: Instant) -> Option<Record> {
		match store.handle([0; 32], REGISTERED, RegistryRequest::Lookup { id }, now) {
			RegistryResponse::Found(record) => record,
			other => panic!("unexpected {other:?}"),
		}
	}

	#[test]
	fn register_then_lookup_returns_the_callers_record() {
		let mut store = Store::default();
		let now = Instant::now();

		let response = store.handle([1; 32], REGISTERED, register(5, &[]), now);

		assert_eq!(response, RegistryResponse::Registered { ttl_secs: 300 });
		let record = lookup(&mut store, [1; 32], now + Duration::from_secs(100)).unwrap();
		assert_eq!(record.id, [1; 32]);
		assert_eq!(record.seq, 5);
		assert_eq!(record.relay_url.as_deref(), Some("https://relay.example.com"));
		assert_eq!(record.expires_in_secs, 200);
	}

	#[test]
	fn records_expire_after_their_ttl() {
		let mut store = Store::default();
		let now = Instant::now();
		store.handle([1; 32], REGISTERED, register(1, &[]), now);

		assert!(lookup(&mut store, [1; 32], now + Duration::from_secs(301)).is_none());
		assert_eq!(store.len(), 0);
	}

	#[test]
	fn ttl_is_capped() {
		let mut store = Store::default();
		let request = RegistryRequest::Register {
			seq: 1,
			relay_url: None,
			direct: vec![],
			namespaces: vec![],
			ttl_secs: 86_400,
		};

		let response = store.handle([1; 32], REGISTERED, request, Instant::now());

		assert_eq!(response, RegistryResponse::Registered { ttl_secs: REGISTRY_MAX_TTL_SECS });
	}

	#[test]
	fn older_or_equal_seq_is_stale() {
		let mut store = Store::default();
		let now = Instant::now();
		store.handle([1; 32], REGISTERED, register(5, &[]), now);

		for seq in [4, 5] {
			let response = store.handle([1; 32], REGISTERED, register(seq, &[]), now);
			assert_eq!(response, RegistryResponse::Err(RegistryError::StaleSeq));
		}
		let response = store.handle([1; 32], REGISTERED, register(6, &[]), now);
		assert_eq!(response, RegistryResponse::Registered { ttl_secs: 300 });
	}

	#[test]
	fn unregister_removes_only_the_callers_record() {
		let mut store = Store::default();
		let now = Instant::now();
		store.handle([1; 32], REGISTERED, register(1, &[]), now);
		store.handle([2; 32], REGISTERED, register(1, &[]), now);

		let response = store.handle([1; 32], REGISTERED, RegistryRequest::Unregister, now);

		assert_eq!(response, RegistryResponse::Ok);
		assert!(lookup(&mut store, [1; 32], now).is_none());
		assert!(lookup(&mut store, [2; 32], now).is_some());
	}

	#[test]
	fn namespaces_need_their_role() {
		let mut store = Store::default();
		let now = Instant::now();
		let cases = [
			(REGISTERED, HOLDERS, RegistryResponse::Err(RegistryError::Forbidden)),
			(PROCESSOR, HOLDERS, RegistryResponse::Registered { ttl_secs: 300 }),
			(PROCESSOR, "jobs/1", RegistryResponse::Err(RegistryError::UnknownNamespace)),
			(PROCESSOR, "secret-holder/x", RegistryResponse::Err(RegistryError::UnknownNamespace)),
			(PROCESSOR, "Not Valid", RegistryResponse::Err(RegistryError::Invalid)),
		];
		for (seq, (roles, ns, expected)) in (1..).zip(cases) {
			assert_eq!(store.handle([1; 32], roles, register(seq, &[ns]), now), expected, "{ns}");
		}
	}

	#[test]
	fn oversized_records_are_invalid() {
		let mut store = Store::default();
		let now = Instant::now();
		let addr = "203.0.113.7:7843".parse().unwrap();
		let requests = [
			RegistryRequest::Register {
				seq: 1,
				relay_url: None,
				direct: vec![addr; MAX_DIRECT_ADDRS + 1],
				namespaces: vec![],
				ttl_secs: 300,
			},
			RegistryRequest::Register {
				seq: 1,
				relay_url: Some(format!("https://{}", "a".repeat(MAX_RELAY_URL_LEN))),
				direct: vec![],
				namespaces: vec![],
				ttl_secs: 300,
			},
			RegistryRequest::Register {
				seq: 1,
				relay_url: None,
				direct: vec![],
				namespaces: vec![],
				ttl_secs: 0,
			},
		];
		for request in requests {
			let response = store.handle([1; 32], PROCESSOR, request, now);
			assert_eq!(response, RegistryResponse::Err(RegistryError::Invalid));
		}
	}

	#[test]
	fn new_records_beyond_capacity_are_refused() {
		let mut store = Store::with_capacity(1);
		let now = Instant::now();
		store.handle([1; 32], REGISTERED, register(1, &[]), now);

		let response = store.handle([2; 32], REGISTERED, register(1, &[]), now);
		assert_eq!(response, RegistryResponse::Err(RegistryError::Busy));
		// The endpoint already stored can still update its record.
		let response = store.handle([1; 32], REGISTERED, register(2, &[]), now);
		assert_eq!(response, RegistryResponse::Registered { ttl_secs: 300 });
	}

	#[test]
	fn discover_pages_through_every_member_once() {
		let mut store = Store::default();
		let now = Instant::now();
		for i in 0..25u8 {
			store.handle([i; 32], PROCESSOR, register(1, &[HOLDERS]), now);
		}
		store.handle([99; 32], PROCESSOR, register(1, &[]), now);

		let mut seen = HashSet::new();
		let mut cursor = None;
		loop {
			let request =
				RegistryRequest::Discover { namespace: HOLDERS.into(), cursor, limit: 10 };
			let RegistryResponse::Discovered { records, cursor: next } =
				store.handle([0; 32], REGISTERED, request, now)
			else {
				panic!("unexpected response");
			};
			assert!(records.len() <= 10);
			for record in records {
				assert!(seen.insert(record.id), "returned twice");
			}
			match next {
				Some(next) => cursor = Some(next),
				None => break,
			}
		}
		assert_eq!(seen, (0..25u8).map(|i| [i; 32]).collect());
	}

	#[test]
	fn discover_checks_namespace_and_limit() {
		let mut store = Store::default();
		let now = Instant::now();
		let discover = |namespace: &str, limit| RegistryRequest::Discover {
			namespace: namespace.into(),
			cursor: None,
			limit,
		};

		let cases = [
			(discover("jobs/1", 10), RegistryError::UnknownNamespace),
			(discover(HOLDERS, 0), RegistryError::Invalid),
			(discover(HOLDERS, MAX_DISCOVER_LIMIT + 1), RegistryError::Invalid),
		];
		for (request, error) in cases {
			let response = store.handle([0; 32], REGISTERED, request, now);
			assert_eq!(response, RegistryResponse::Err(error));
		}
		let response = store.handle([0; 32], REGISTERED, discover(HOLDERS, 10), now);
		assert_eq!(response, RegistryResponse::Discovered { records: vec![], cursor: None });
	}
}
