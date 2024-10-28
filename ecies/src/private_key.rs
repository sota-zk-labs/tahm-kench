use curve25519_dalek::Scalar;
use rand::Rng;

use crate::public_key::PublicKey;

pub struct PrivateKey {
    key: Scalar,
}

impl PrivateKey {
    pub fn from_rng(rng: &mut impl Rng) -> Self {
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes);
        Self::from_bytes(bytes)
    }

    pub fn from_hex(hex_str: &str) -> Self {
        Self::from_bytes(hex::decode(hex_str).unwrap().try_into().unwrap())
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            key: Scalar::from_bytes_mod_order(bytes),
        }
    }

    pub fn to_public_key(&self) -> PublicKey {
        PublicKey::from_point(&self.key * &curve25519_dalek::constants::ED25519_BASEPOINT_POINT)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.key.to_bytes()
    }
    
    pub fn scalar(&self) -> &Scalar {
        &self.key
    }
}
