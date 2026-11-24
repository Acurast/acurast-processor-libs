//! Expected values generated with polkadot-js (`@polkadot/keyring`, sr25519): the same vectors as
//! the apps' own tests.

use super::*;

struct Vector {
    entropy: &'static str,
    mini_secret: &'static str,
    public_keys: &'static [(&'static str, &'static str)],
}

const VECTORS: &[Vector] = &[
    // abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
    Vector {
        entropy: "00000000000000000000000000000000",
        mini_secret: "4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2",
        public_keys: &[
            (
                "",
                "66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972",
            ),
            (
                "//44//434//0/0/0",
                "1af9a96dc6ffa745a9f43f10727f2bd70940af404e19d6e0319335ec913b4a0d",
            ),
            (
                "//1",
                "367af3b0d5bae948f910bf5365da6cb7639b0e4b6c3b0131f7d36637556a833f",
            ),
            (
                "//2",
                "582e53b26d1138af19d141d469ec45b9370b0339d5f6d086b03ee7d77f8ca846",
            ),
            (
                "//acurast/soft//7",
                "1446225a2afba61d9fed932be157efd7cd4af7b11739e3990714fceab9951375",
            ),
        ],
    },
    // bottom drive obey lake curtain smoke basket hold race lonely fit walk
    Vector {
        entropy: "1a486a5fbe53639984cb64b070755f7b",
        mini_secret: "fac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e",
        public_keys: &[
            (
                "",
                "46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a",
            ),
            (
                "//44//434//0/0/0",
                "32f1dac6df4f73b4c743ac9d583b8445c3c173e16a73cceb7d27fa3a10b5041b",
            ),
            (
                "//1",
                "b606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48",
            ),
            (
                "//2",
                "46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a",
            ),
            (
                "//acurast/soft//7",
                "e05fd6f4374c3f96187555bf3b2678d06046bf1342cc22d2cb2a0b9b99e45856",
            ),
        ],
    },
    // void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing
    // screen patrol group space point ten exist slush involve unfold
    Vector {
        entropy: "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
        mini_secret: "7316d66bc0ad15affde3eee7903fdbd893f5089961c9cd53144bc4b3f16e965b",
        public_keys: &[
            (
                "",
                "0c4c666bdae258d7e45b7fe632777ecc25ba906140282ceeb0c204dc1de94d5a",
            ),
            (
                "//44//434//0/0/0",
                "6801f603273d9febd71696f0ea58ded3506d9fbc5c0902bc9a7fc18aa5401a27",
            ),
            (
                "//1",
                "dad6a8d79ee51b2c969be4d9b6440c3f3adaea7453c2728a80d0d0fe6c9a781a",
            ),
            (
                "//2",
                "da55f8dd028aef5d899b7945e49bddd88abb9de1e407201749ddce6311b0da05",
            ),
            (
                "//acurast/soft//7",
                "36a56b34e387cdf6ce4b6de784abaafd09405c91e2f27d569df0dd4b06d2e374",
            ),
        ],
    },
];

fn bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex).unwrap()
}

#[test]
fn mini_secret_key_matches_polkadot_js() {
    for vector in VECTORS {
        let mini_secret = mini_secret_key(&bytes(vector.entropy)).unwrap();
        assert_eq!(hex::encode(mini_secret.to_bytes()), vector.mini_secret);
    }
}

#[test]
fn derives_the_same_public_keys_as_polkadot_js() {
    for vector in VECTORS {
        for (path, public_key) in vector.public_keys {
            let keypair = Sr25519KeyPair::new(bytes(vector.entropy), path.to_string()).unwrap();
            assert_eq!(
                hex::encode(keypair.public_key()),
                *public_key,
                "path '{path}'"
            );
        }
    }
}

#[test]
fn signatures_verify_against_the_public_key() {
    let keypair =
        Sr25519KeyPair::new(bytes(VECTORS[0].entropy), "//44//434//0/0/0".to_string()).unwrap();
    let message = b"<Bytes>hello</Bytes>".to_vec();

    let signature = keypair.sign(message.clone());

    assert_eq!(signature.len(), 64);
    assert!(sr25519_verify(
        keypair.public_key(),
        message,
        signature.clone()
    ));
    assert!(!sr25519_verify(
        keypair.public_key(),
        b"other".to_vec(),
        signature
    ));
}

#[test]
fn verifies_a_polkadot_js_signature() {
    let public_key = bytes(VECTORS[0].public_keys[0].1);
    let signature = bytes(concat!(
        "a4d7ad88bb21c1522b1ee80179f7de26fcdb63f6dcae06b0ac97f89357f3f931",
        "392377a4cd320f4447b84d238876c764a386be2b0edc776b891a19926063fd8e",
    ));

    assert!(sr25519_verify(public_key, b"hello".to_vec(), signature));
}

#[test]
fn verify_rejects_malformed_keys_and_signatures() {
    assert!(!sr25519_verify(vec![0; 31], b"hello".to_vec(), vec![0; 64]));
    assert!(!sr25519_verify(
        bytes(VECTORS[0].public_keys[0].1),
        b"hello".to_vec(),
        vec![0; 63]
    ));
}

#[test]
fn rejects_malformed_paths() {
    for path in ["44", "///", "//44/", "//44///pw"] {
        assert!(
            matches!(
                Sr25519KeyPair::new(bytes(VECTORS[0].entropy), path.to_string()),
                Err(Sr25519Error::InvalidDerivationPath { .. })
            ),
            "path '{path}'"
        );
    }
}

#[test]
fn rejects_invalid_entropy() {
    for entropy in [vec![0u8; 15], vec![0u8; 17], vec![0u8; 36]] {
        assert!(matches!(
            Sr25519KeyPair::new(entropy, String::new()),
            Err(Sr25519Error::InvalidEntropy { .. })
        ));
    }
}
