//! sr25519 keys of recovery phrases from Substrate wallets (polkadot-js, SubWallet, Talisman,
//! AirGap Vault), for the apps that import them.
//!
//! Built on the reference implementations: `schnorrkel` for keys and signatures, and Parity's
//! `substrate-bip39` for the mini secret key of a phrase. Derivation paths follow `sp-core`.

use std::sync::Arc;

use blake2::{Blake2b, Digest, digest::consts::U32};
use parity_scale_codec::Encode;
use schnorrkel::{
    ExpansionMode, Keypair, MiniSecretKey, PublicKey, Signature,
    derive::{ChainCode, Derivation},
};

uniffi::setup_scaffolding!();

/// The signing context of Substrate: every sr25519 signature on chain is made in it.
const SIGNING_CONTEXT: &[u8] = b"substrate";

const CHAIN_CODE_LENGTH: usize = 32;

/// Errors returned to the app.
#[derive(Debug, PartialEq, thiserror::Error, uniffi::Error)]
pub enum Sr25519Error {
    /// The entropy is not the entropy of a BIP-39 recovery phrase (16 to 32 bytes, multiple of 4).
    #[error("invalid entropy: {reason}")]
    InvalidEntropy { reason: String },
    /// The path is not a derivation path such as `//44//434//0/0/0`.
    #[error("invalid derivation path: {path}")]
    InvalidDerivationPath { path: String },
    /// The bytes are not a key pair serialized with [`Sr25519KeyPair::to_bytes`].
    #[error("invalid key pair: {reason}")]
    InvalidKeyPair { reason: String },
}

/// An sr25519 key pair derived from a recovery phrase the way Substrate wallets derive it.
#[derive(uniffi::Object)]
pub struct Sr25519KeyPair {
    keypair: Keypair,
}

#[uniffi::export]
impl Sr25519KeyPair {
    /// Derives the key pair at `path` (`""` for the root key, `//` for a hard junction, `/` for a
    /// soft one) from the BIP-39 `entropy` of a recovery phrase.
    #[uniffi::constructor]
    pub fn new(entropy: Vec<u8>, path: String) -> Result<Arc<Self>, Sr25519Error> {
        let junctions = parse_derivation_path(&path)?;
        let root = mini_secret_key(&entropy)?.expand(ExpansionMode::Ed25519);
        let secret = junctions
            .into_iter()
            .fold(root, |parent, junction| match junction {
                Junction::Hard(chain_code) => parent
                    .hard_derive_mini_secret_key(Some(ChainCode(chain_code)), b"")
                    .0
                    .expand(ExpansionMode::Ed25519),
                Junction::Soft(chain_code) => {
                    parent.derived_key_simple(ChainCode(chain_code), []).0
                }
            });

        Ok(Arc::new(Self {
            keypair: secret.to_keypair(),
        }))
    }

    /// Restores a key pair serialized with [`Sr25519KeyPair::to_bytes`].
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Arc<Self>, Sr25519Error> {
        let keypair = Keypair::from_bytes(&bytes).map_err(|e| Sr25519Error::InvalidKeyPair {
            reason: e.to_string(),
        })?;

        Ok(Arc::new(Self { keypair }))
    }

    /// The key pair as 96 bytes, to store it: secret key (64 bytes) and public key (32 bytes).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.keypair.to_bytes().to_vec()
    }

    /// The 32-byte public key, which is also the Substrate account id.
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public.to_bytes().to_vec()
    }

    /// Signs `message` as is in the Substrate signing context: a 64-byte signature.
    pub fn sign(&self, message: Vec<u8>) -> Vec<u8> {
        self.keypair
            .sign_simple(SIGNING_CONTEXT, &message)
            .to_bytes()
            .to_vec()
    }
}

/// Whether `signature` is a valid Substrate sr25519 signature of `message` by `public_key`.
#[uniffi::export]
pub fn sr25519_verify(public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> bool {
    let (Ok(public_key), Ok(signature)) = (
        PublicKey::from_bytes(&public_key),
        Signature::from_bytes(&signature),
    ) else {
        return false;
    };

    public_key
        .verify_simple(SIGNING_CONTEXT, &message, &signature)
        .is_ok()
}

/// The mini secret key of a recovery phrase: PBKDF2 over its entropy (not over its words, unlike
/// the BIP-39 seed), as polkadot-js and `sp-core` compute it.
fn mini_secret_key(entropy: &[u8]) -> Result<MiniSecretKey, Sr25519Error> {
    substrate_bip39::mini_secret_from_entropy(entropy, "").map_err(|e| {
        Sr25519Error::InvalidEntropy {
            reason: format!("{e:?}"),
        }
    })
}

enum Junction {
    Hard([u8; CHAIN_CODE_LENGTH]),
    Soft([u8; CHAIN_CODE_LENGTH]),
}

/// Parses a derivation path without phrase or password (`sp-core` would read `///` as the start of
/// a password): every junction is `//` or `/` followed by a non-empty code.
fn parse_derivation_path(path: &str) -> Result<Vec<Junction>, Sr25519Error> {
    let invalid = || Sr25519Error::InvalidDerivationPath {
        path: path.to_string(),
    };
    let mut junctions = Vec::new();
    let mut rest = path;
    while !rest.is_empty() {
        let (is_hard, tail) = match rest.strip_prefix("//") {
            Some(tail) => (true, tail),
            None => (false, rest.strip_prefix('/').ok_or_else(invalid)?),
        };
        let end = tail.find('/').unwrap_or(tail.len());
        let code = &tail[..end];
        if code.is_empty() {
            return Err(invalid());
        }

        let chain_code = chain_code(code);
        junctions.push(if is_hard {
            Junction::Hard(chain_code)
        } else {
            Junction::Soft(chain_code)
        });
        rest = &tail[end..];
    }

    Ok(junctions)
}

/// The chain code of a junction, as `sp-core`'s `DeriveJunction` computes it: the SCALE encoding
/// of the code (a `u64` if it is a number, a string otherwise), zero-padded, or its BLAKE2b-256
/// hash if it is longer than a chain code.
fn chain_code(code: &str) -> [u8; CHAIN_CODE_LENGTH] {
    let encoded = match code.parse::<u64>() {
        Ok(number) => number.encode(),
        Err(_) => code.encode(),
    };

    let mut chain_code = [0u8; CHAIN_CODE_LENGTH];
    if encoded.len() > CHAIN_CODE_LENGTH {
        chain_code.copy_from_slice(&Blake2b::<U32>::digest(&encoded));
    } else {
        chain_code[..encoded.len()].copy_from_slice(&encoded);
    }

    chain_code
}

#[cfg(test)]
mod tests;
