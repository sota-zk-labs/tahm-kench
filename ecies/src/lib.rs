use std::marker::PhantomData;
use crate::private_key::PrivateKey;
use crate::public_key::PublicKey;
use crate::symmetric_encryption::scheme::SymmetricEncryptionScheme;
use rand::Rng;
use crate::symmetric_encryption::simple::SimpleSE;

pub mod ecc;
pub mod private_key;
pub mod public_key;
pub mod symmetric_encryption;
pub mod types;
pub mod utils;

pub struct Ecies<S: SymmetricEncryptionScheme = SimpleSE> {
    pvk: PrivateKey,
    pbk: PublicKey,
    phantom_data: PhantomData<S>
}

impl<S: SymmetricEncryptionScheme> Ecies<S> {
    pub fn from_pvk(pvk: PrivateKey) -> Self {
        let pbk = pvk.to_public_key();
        Ecies {
            pvk,
            pbk,
            phantom_data: PhantomData,
        }
    }

    pub fn encrypt(&self, rng: &mut impl Rng, pbk: &PublicKey, plaintext: &[u8]) -> Vec<u8> {
        let scheme = S::new(Self::get_symmetric_key(&self.pvk, pbk));
        let mut res = self.pbk.to_bytes().to_vec();
        res.extend(&scheme.encrypt(rng, plaintext));
        res
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let sender_pbk = PublicKey::from_bytes(ciphertext[..32].to_vec());
        let scheme = S::new(Self::get_symmetric_key(&self.pvk, &sender_pbk));
        scheme.decrypt(&ciphertext[32..])
    }

    pub fn borrow_pbk(&self) -> &PublicKey {
        &self.pbk
    }

    fn get_symmetric_key(pvk: &PrivateKey, ephemeral_pbk: &PublicKey) -> Vec<u8> {
        let pvk_num = u128::from_be_bytes(pvk.to_bytes().try_into().unwrap());
        let curve = pvk.curve();
        #[cfg(test)]
        println!("cycle-tracker-start: get_symmetric_key");
        let res = curve
            .mul(&ephemeral_pbk.to_point(curve.mont_space()), pvk_num)
            .to_bytes(curve.mont_space());
        #[cfg(test)]
        println!("cycle-tracker-end: get_symmetric_key");
        res
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use crate::Ecies;
    use crate::private_key::PrivateKey;
    use crate::symmetric_encryption::simple::SimpleSE;

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
