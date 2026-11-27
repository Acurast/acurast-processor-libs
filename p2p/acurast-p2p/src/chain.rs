//! Reads of the P2P runtime API (`P2pApi`) through [`Chain`].

use parity_scale_codec::Decode;

use crate::{Error, Relay, platform::Chain};

/// On-chain relay node, as encoded by the P2P pallet.
#[derive(Decode)]
struct RelayNode {
    url: Vec<u8>,
    quic_port: Option<u16>,
    _country: u16,
    _operator: [u8; 32],
}

/// The runtime's version, as returned by `Core_version`; only the fields read here.
#[derive(Decode)]
struct RuntimeVersion {
    _spec_name: Vec<u8>,
    _impl_name: Vec<u8>,
    _authoring_version: u32,
    spec_version: u32,
    _impl_version: u32,
    apis: Vec<([u8; 8], u32)>,
}

/// Id of the `P2pApi` runtime API: the first 8 bytes of its name's BLAKE2b hash.
const P2P_API_ID: [u8; 8] = [0xd0, 0xd6, 0x24, 0x0d, 0x9a, 0x9c, 0xa0, 0xfc];

/// Whether the chain's runtime has the P2P layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSupport {
    /// Version of the runtime that was asked, to ask again only once it changes.
    pub spec_version: u32,
    /// Whether the runtime offers the P2P runtime API, without which no endpoint can register.
    pub p2p: bool,
}

/// Reads from the chain's runtime whether it has the P2P layer.
pub async fn runtime_support(chain: &dyn Chain) -> Result<RuntimeSupport, Error> {
    let version: RuntimeVersion = call(chain, "Core_version", Vec::new()).await?;
    Ok(RuntimeSupport {
        spec_version: version.spec_version,
        p2p: version.apis.iter().any(|(id, _)| *id == P2P_API_ID),
    })
}

async fn call<R: Decode>(chain: &dyn Chain, method: &str, args: Vec<u8>) -> Result<R, Error> {
    let output = chain.state_call(method.into(), args).await?;
    R::decode(&mut &output[..]).map_err(|e| Error::InvalidChainData(format!("{method}: {e}")))
}

/// Endpoints registered by this processor's account.
pub(crate) async fn own_endpoints(chain: &dyn Chain) -> Result<Vec<[u8; 32]>, Error> {
    call(chain, "P2pApi_endpoints", chain.account()).await
}

/// Account that registered the endpoint `id`, SCALE-encoded.
pub(crate) async fn endpoint_owner(
    chain: &dyn Chain,
    id: &[u8; 32],
) -> Result<Option<Vec<u8>>, Error> {
    let owner: Option<[u8; 32]> = call(chain, "P2pApi_endpoint_owner", id.to_vec()).await?;
    Ok(owner.map(|owner| owner.to_vec()))
}

/// Relay nodes listed on chain. Entries with an invalid id or URL are skipped.
pub(crate) async fn relay_nodes(chain: &dyn Chain) -> Result<Vec<Relay>, Error> {
    let nodes: Vec<([u8; 32], RelayNode)> = call(chain, "P2pApi_relay_nodes", Vec::new()).await?;
    Ok(nodes
        .into_iter()
        .filter_map(|(id, node)| Relay::from_chain(&id, &node.url, node.quic_port).ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformError;

    const ALICE: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
    const ENDPOINT: &str = "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c";

    /// Answers recorded from a local network running the Canary runtime, after //Alice registered
    /// `ENDPOINT` and Root listed the relay `https://relay.local.test:8443` (QUIC port 7842).
    struct RecordedChain;

    #[async_trait::async_trait]
    impl Chain for RecordedChain {
        fn account(&self) -> Vec<u8> {
            hex::decode(ALICE).unwrap()
        }

        async fn genesis_hash(&self) -> Result<Vec<u8>, PlatformError> {
            unreachable!()
        }

        async fn state_call(
            &self,
            method: String,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, PlatformError> {
            let answer = match method.as_str() {
                "P2pApi_endpoints" => {
                    assert_eq!(hex::encode(args), ALICE);
                    format!("04{ENDPOINT}")
                }
                // The same relay twice: once with a valid node id, once with an id that is no
                // Ed25519 key.
                "P2pApi_relay_nodes" => {
                    let node = format!(
                        "74{}01a21ef402{ALICE}",
                        hex::encode("https://relay.local.test:8443")
                    );
                    format!("08{ENDPOINT}{node}{}{node}", "02".repeat(32))
                }
                "Core_version" => DEVNET_VERSION.to_owned(),
                _ => unreachable!(),
            };
            Ok(hex::decode(answer).unwrap())
        }

        async fn register_endpoint(&self, _: Vec<u8>, _: Vec<u8>) -> Result<(), PlatformError> {
            unreachable!()
        }
    }

    /// `Core_version` of the devnet (runtime 170, with the P2P layer) and of Canary (without it).
    const DEVNET_VERSION: &str = "44616375726173742d70617261636861696e44616375726173742d70617261636861696e03000000aa0000000200000044dd718d5cc53262d401000000d7bdd8a272ca0d650200000004e70521a0d3d2f802000000075f8cd374350e8401000000df6acb689907609b0500000037e397fc7c91f5e402000000ccd9de6396c899ca0100000040fe3ad401f8959a06000000d2bc9897eed08f1503000000f78b278be53f454c02000000ab3c0572291feb8b02000000bc9d89904f5b923f0100000037c8bb1350a9a2a804000000f3ff14d5ab52705903000000d0d6240d9a9ca0fc01000000ea93e3f16f3d696203000000fbc577b9d747efd6010000000100000001";
    const CANARY_VERSION: &str = "44616375726173742d70617261636861696e44616375726173742d70617261636861696e01000000410000000100000040dd718d5cc53262d401000000d7bdd8a272ca0d650200000004e70521a0d3d2f802000000075f8cd374350e8401000000df6acb689907609b0500000037e397fc7c91f5e402000000ccd9de6396c899ca0100000040fe3ad401f8959a06000000d2bc9897eed08f1503000000f78b278be53f454c02000000ab3c0572291feb8b02000000bc9d89904f5b923f0100000037c8bb1350a9a2a804000000f3ff14d5ab52705903000000ea93e3f16f3d696203000000fbc577b9d747efd6010000000100000001";

    #[test]
    fn the_p2p_api_id_is_the_hash_of_its_name() {
        use blake2::{Blake2b, Digest, digest::consts::U8};
        assert_eq!(Blake2b::<U8>::digest(b"P2pApi")[..], P2P_API_ID);
    }

    #[tokio::test]
    async fn tells_whether_the_runtime_has_the_p2p_layer() {
        let devnet = runtime_support(&RecordedChain).await.unwrap();
        assert_eq!(
            devnet,
            RuntimeSupport {
                spec_version: 170,
                p2p: true
            }
        );

        let canary =
            RuntimeVersion::decode(&mut &hex::decode(CANARY_VERSION).unwrap()[..]).unwrap();
        assert!(!canary.apis.iter().any(|(id, _)| *id == P2P_API_ID));
    }

    #[tokio::test]
    async fn decodes_runtime_answers() {
        let endpoints = own_endpoints(&RecordedChain).await.unwrap();
        assert_eq!(
            endpoints,
            vec![<[u8; 32]>::try_from(hex::decode(ENDPOINT).unwrap()).unwrap()]
        );

        let relays = relay_nodes(&RecordedChain).await.unwrap();
        assert_eq!(relays.len(), 1, "the relay with an invalid id is skipped");
        assert_eq!(hex::encode(relays[0].id.as_bytes()), ENDPOINT);
        assert_eq!(relays[0].url.as_str(), "https://relay.local.test:8443/");
        assert_eq!(relays[0].quic_port, Some(7842));
    }
}
