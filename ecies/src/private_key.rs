use crate::ecc::curve::Ecc;
use rand::{Rng};
use crate::public_key::PublicKey;

pub struct PrivateKey {
    key: Vec<u8>,
    // secp112r1
    curve: Ecc,
}

impl PrivateKey {
    pub fn from_rng(rng: &mut impl Rng) -> Self {
        let n_bits = 112;
        let mut bytes = vec![0; 16];
        rng.fill_bytes(&mut bytes[16 - n_bits / 8..]);
        Self::from_bytes(bytes)
    }

    pub fn from_hex(hex_str: &str) -> Self {
        Self::from_bytes(hex::decode(hex_str).unwrap())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            key: bytes,
            curve: Ecc::new(
                0xdb7c2abf62e35e668076bead208b,
                0xdb7c2abf62e35e668076bead2088,
                // 0x659ef8ba043916eede8911702b22,
                (
                    0x09487239995a5ee76b55f9c2f098,
                    0xa89ce5af8724c0a23e0e0ff77500,
                ),
                0xdb7c2abf62e35e7628dfac6561c5,
                // 0x01,
            ),
        }
    }

    pub fn to_public_key(&self) -> PublicKey {
        self.curve.get_public_key(self.key.clone())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.key.clone()
    }

    pub fn curve(&self) -> &Ecc {
        &self.curve
    }
}
