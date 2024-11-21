use std::marker::PhantomData;

use rand::Rng;
use secp256k1::{All, Scalar, Secp256k1};

use crate::private_key::PrivateKey;
use crate::public_key::PublicKey;
use crate::symmetric_encryption::scheme::SymmetricEncryptionScheme;
use crate::symmetric_encryption::simple::SimpleSE;

pub mod private_key;
pub mod public_key;
pub mod symmetric_encryption;
pub mod utils;

pub struct Ecies<S: SymmetricEncryptionScheme = SimpleSE> {
    pvk: PrivateKey,
    pbk: PublicKey,
    secp: Secp256k1<All>,
    phantom_data: PhantomData<S>,
}

impl<S: SymmetricEncryptionScheme> Ecies<S> {
    pub fn from_pvk(pvk: PrivateKey) -> Self {
        let pbk = pvk.to_public_key();
        Ecies {
            pvk,
            pbk,
            secp: Secp256k1::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn encrypt(&self, rng: &mut impl Rng, pbk: &PublicKey, plaintext: &[u8]) -> Vec<u8> {
        let scheme = S::new(self.get_symmetric_key(&self.pvk, pbk));
        let mut res = self.pbk.to_bytes().to_vec();
        res.extend(&scheme.encrypt(rng, plaintext));
        res
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let sender_pbk = PublicKey::from_bytes(&ciphertext[..65]);
        let scheme = S::new(self.get_symmetric_key(&self.pvk, &sender_pbk));
        scheme.decrypt(&ciphertext[65..])
    }

    pub fn borrow_pbk(&self) -> &PublicKey {
        &self.pbk
    }

    fn get_symmetric_key(&self, pvk: &PrivateKey, ephemeral_pbk: &PublicKey) -> Vec<u8> {
        ephemeral_pbk
            .key
            .mul_tweak(&self.secp, &Scalar::from(pvk.key))
            .unwrap()
            .serialize_uncompressed()
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;

    use crate::private_key::PrivateKey;
    use crate::symmetric_encryption::simple::SimpleSE;
    use crate::Ecies;

    #[test]
    fn test_ecies() {
        let rng = &mut OsRng;
        let sender = Ecies::<SimpleSE>::from_pvk(PrivateKey::from_rng(rng));
        let receiver = Ecies::<SimpleSE>::from_pvk(PrivateKey::from_rng(rng));
        let plaintext = b"Hello, world!";
        let encrypted_text = sender.encrypt(rng, receiver.borrow_pbk(), plaintext);
        let decrypted = receiver.decrypt(&encrypted_text);
        assert_eq!(plaintext, decrypted.as_slice());
    }
}
