use ecies::SecretKey;
use serde::{Deserialize, Serialize};

pub const PVK_HEX: &str = include_str!("../pvk");

#[derive(Deserialize, Serialize)]
pub struct AuctionData {
    pub bidders: Vec<Bidder>,
    pub id: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bidder {
    pub encrypted_amount: Vec<u8>,
    pub address: Vec<u8>,
}

pub fn decrypt_bidder_data(pvk: &SecretKey, bidder: &Bidder) -> u128 {
    let data = String::from_utf8(
        ecies::decrypt(&pvk.serialize(), &bidder.encrypted_amount)
            .expect("failed to decrypt"),
    )
        .unwrap();
    data.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::{AuctionData, Bidder, PVK_HEX};
    // use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
    // use rsa::rand_core::OsRng;
    // use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
    use sp1_sdk::{ProverClient, SP1Stdin};
    // use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;
    use ecies::{PublicKey, SecretKey};
    use rand::rngs::{OsRng};
    // use ecies_ed25519::generate_keypair;
    // use rand::{CryptoRng, RngCore};

    #[test]
    fn test_sp1_prover() {
        // compile main
        let output = Command::new("cargo")
            .args(["prove", "build"])
            .output()
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(output.stdout.as_slice()));
        let elf = {
            let mut buffer = Vec::new();
            File::open("./elf/riscv32im-succinct-zkvm-elf")
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();
            buffer
        };

        // let mut rng = rand::thread_rng();
        let (_, pbk) = get_key();

        let mut stdin = SP1Stdin::new();
        stdin.write(&AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&2, &pbk),
                    address: vec![5; 32],
                },
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&1, &pbk),
                    address: vec![1; 32],
                },
            ],
            id: vec![0; 32],
        });

        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf.as_slice());

        let Ok(mut proof) = client.prove(&pk, stdin).run() else {
            println!("Something went wrong!");
            return;
        };

        println!("Proof generated successfully. Verifying proof...");
        client.verify(&proof, &vk).expect("verification failed");
        println!("Proof verified successfully.");

        // println!("{:?}", proof.public_values);
        let hash_data = proof.public_values.read::<[u8; 32]>();
        println!("{:?}", hash_data);
        let winner_addr = proof.public_values.read::<Vec<u8>>();
        println!("{:?}", winner_addr);
        // Todo: validate with data
    }

    #[test]
    fn test_decrypt_data() {
        let (pvk, pbk) = get_key();
        // let mut rng = rand::thread_rng();
        let bidder = Bidder {
            encrypted_amount: encrypt_bidder_amount(&(1e23 as u128), &pbk),
            address: vec![0; 32],
        };
        let amount = super::decrypt_bidder_data(&pvk, &bidder);
        assert_eq!(amount, 1e23 as u128);
    }

    #[test]
    fn test_gen_key() {
        let mut rng = OsRng;
        let pvk = SecretKey::random(&mut rng);
        let pbk = PublicKey::from_secret_key(&pvk);
        let pvk = hex::encode(pvk.serialize());
        let pbk = hex::encode(pbk.serialize());
        println!("Private key: {}", &pvk);
        println!("Public key: {}", &pbk);
        fs::write("pvk", pvk).expect("failed to write private key to file");
        fs::write("pbk", pbk).expect("failed to write public key to file");
    }

    fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
        let data = amount.to_string();
        ecies::encrypt(&pbk.serialize(), data.as_bytes()).expect("failed to encrypt bidder data")
    }

    fn get_key() -> (SecretKey, PublicKey) {
        let pvk = SecretKey::parse_slice(&hex::decode(PVK_HEX).unwrap()).expect("fail to read private key");
        (pvk, PublicKey::from_secret_key(&pvk))
    }
}
