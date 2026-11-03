//! The registry served on a local iroh endpoint, reached by clients over real QUIC connections.

use std::{
	collections::HashMap,
	net::{Ipv4Addr, SocketAddr},
	sync::Arc,
};

use acurast_p2p_proto::{
	frame::{self, HEADER_LEN},
	messages::{RegistryRequest, RegistryResponse},
	EndpointId, ALPN_REGISTRY, ERR_LIMIT, ERR_ROLE, MAX_FRAME, ROLE_REGISTERED,
};
use acurast_p2p_registry::{Registry, RoleSource};
use iroh::{
	endpoint::{presets, ConnectionError, VarInt},
	protocol::Router,
	Endpoint, EndpointAddr, RelayMode, SecretKey,
};

/// Roles from a fixed table; unknown endpoints have none.
struct Roles(HashMap<EndpointId, u8>);

impl RoleSource for Roles {
	fn roles(&self, id: &EndpointId) -> Result<u8, String> {
		Ok(self.0.get(id).copied().unwrap_or(0))
	}
}

async fn local_endpoint(key: &SecretKey, alpns: Vec<Vec<u8>>) -> Endpoint {
	Endpoint::builder(presets::Minimal)
		.secret_key(key.clone())
		.relay_mode(RelayMode::Disabled)
		.alpns(alpns)
		.bind()
		.await
		.unwrap()
}

/// Starts a registry that knows the given registered endpoints.
async fn registry(registered: &[&SecretKey]) -> (Router, EndpointAddr) {
	let roles = registered
		.iter()
		.map(|key| (*key.public().as_bytes(), ROLE_REGISTERED))
		.collect();
	let endpoint = local_endpoint(&SecretKey::generate(), vec![]).await;
	let port = endpoint.bound_sockets()[0].port();
	let addr = EndpointAddr::new(endpoint.id())
		.with_ip_addr(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
	let router = Router::builder(endpoint)
		.accept(ALPN_REGISTRY, Registry::new(Arc::new(Roles(roles))))
		.spawn();
	(router, addr)
}

async fn request(
	connection: &iroh::endpoint::Connection,
	request: &RegistryRequest,
) -> Result<RegistryResponse, String> {
	let (mut send, mut recv) = connection.open_bi().await.map_err(|e| e.to_string())?;
	send.write_all(&frame::encode(request).unwrap())
		.await
		.map_err(|e| e.to_string())?;
	send.finish().map_err(|e| e.to_string())?;
	let bytes = recv.read_to_end(HEADER_LEN + MAX_FRAME).await.map_err(|e| e.to_string())?;
	Ok(frame::decode(&bytes).map_err(|e| e.to_string())?.0)
}

fn register(seq: u64) -> RegistryRequest {
	RegistryRequest::Register {
		seq,
		relay_url: Some("https://relay.example.com".into()),
		direct: vec![],
		namespaces: vec![],
		ttl_secs: 300,
	}
}

fn closed_with(error: ConnectionError, code: u32) -> bool {
	matches!(error, ConnectionError::ApplicationClosed(close) if close.error_code == VarInt::from_u32(code))
}

#[tokio::test]
async fn registered_endpoint_publishes_and_others_find_it() {
	let publisher = SecretKey::generate();
	let resolver = SecretKey::generate();
	let (router, addr) = registry(&[&publisher, &resolver]).await;

	let client = local_endpoint(&publisher, vec![]).await;
	let connection = client.connect(addr.clone(), ALPN_REGISTRY).await.unwrap();
	assert_eq!(
		request(&connection, &register(1)).await.unwrap(),
		RegistryResponse::Registered { ttl_secs: 300 }
	);

	let other = local_endpoint(&resolver, vec![]).await;
	let connection = other.connect(addr, ALPN_REGISTRY).await.unwrap();
	let lookup = RegistryRequest::Lookup { id: *publisher.public().as_bytes() };
	let RegistryResponse::Found(Some(record)) = request(&connection, &lookup).await.unwrap() else {
		panic!("record not found");
	};
	// The id comes from the authenticated connection, not from the request.
	assert_eq!(record.id, *publisher.public().as_bytes());
	assert_eq!(record.relay_url.as_deref(), Some("https://relay.example.com"));

	router.shutdown().await.unwrap();
}

#[tokio::test]
async fn unregistered_endpoint_is_closed() {
	let (router, addr) = registry(&[]).await;
	let client = local_endpoint(&SecretKey::generate(), vec![]).await;

	let connection = client.connect(addr, ALPN_REGISTRY).await.unwrap();

	assert!(request(&connection, &register(1)).await.is_err());
	assert!(closed_with(connection.closed().await, ERR_ROLE));
	router.shutdown().await.unwrap();
}

#[tokio::test]
async fn too_many_requests_close_the_connection() {
	let key = SecretKey::generate();
	let (router, addr) = registry(&[&key]).await;
	let client = local_endpoint(&key, vec![]).await;
	let connection = client.connect(addr, ALPN_REGISTRY).await.unwrap();

	for seq in 1..=10 {
		request(&connection, &register(seq)).await.unwrap();
	}

	assert!(request(&connection, &register(11)).await.is_err());
	assert!(closed_with(connection.closed().await, ERR_LIMIT));
	router.shutdown().await.unwrap();
}
