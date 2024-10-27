use rand::{Rng};
use secp256k1::{Secp256k1, SecretKey};
use crate::public_key::PublicKey;

pub struct PrivateKey {
    pub key: SecretKey
}

impl PrivateKey {
    pub fn from_rng(rng: &mut impl Rng) -> Self {
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }

    pub fn from_hex(hex_str: &str) -> Self {
        Self::from_bytes(&hex::decode(hex_str).unwrap())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            key: SecretKey::from_slice(bytes).unwrap()
        }
    }

    pub fn to_public_key(&self) -> PublicKey {
        PublicKey::from_bytes(&self.key.public_key(&Secp256k1::new()).serialize_uncompressed())
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.key.secret_bytes()
    }
}
