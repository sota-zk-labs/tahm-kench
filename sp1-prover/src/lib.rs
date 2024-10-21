use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use serde::{Deserialize, Serialize};

pub const PVK_PEM: &str = include_str!("../pvk.pem");

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

pub fn decrypt_bidder_data(pvk: &RsaPrivateKey, bidder: &Bidder) -> u128 {
    let data = String::from_utf8(
        pvk.decrypt(Pkcs1v15Encrypt, &bidder.encrypted_amount)
            .expect("failed to decrypt"),
    )
    .unwrap();
    data.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{AuctionData, Bidder, PVK_PEM};
    use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::rand_core::OsRng;
    use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
    use sp1_sdk::{ProverClient, SP1Stdin};
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;

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

        let pvk = RsaPrivateKey::from_pkcs8_pem(PVK_PEM)
            .expect("missing private key to encode bidder data");
        let pbk = pvk.to_public_key();

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
        let pvk = RsaPrivateKey::from_pkcs8_pem(PVK_PEM).unwrap();
        let pbk = pvk.to_public_key();
        let bidder = Bidder {
            encrypted_amount: encrypt_bidder_amount(&(1e23 as u128), &pbk),
            address: vec![0; 32],
        };
        let amount = super::decrypt_bidder_data(&pvk, &bidder);
        assert_eq!(amount, 1e23 as u128);
    }

    #[test]
    fn test_gen_key() {
        // Generate a 2048-bit RSA private key
        let mut rng = OsRng;
        let bit_size = env::var("BIT_SIZE").unwrap_or("90".to_string()).parse().unwrap();
        println!("Bit size: {}", bit_size);
        let private_key =
            RsaPrivateKey::new(&mut rng, bit_size)
                .expect("failed to generate a key");

        // Extract the public key from the private key
        let public_key = RsaPublicKey::from(&private_key);

        // Print the private key in PEM format
        let private_key_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("failed to encode private key to PEM")
            .to_string();
        std::fs::write("pvk.pem", &private_key_pem).expect("failed to write private key to file"); // write this to file
        println!("Private Key:\n{}", private_key_pem);

        // Print the public key in PEM format
        let public_key_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .expect("failed to encode public key to PEM");
        std::fs::write("pbk.pem", &public_key_pem).expect("failed to write public key to file");
        println!("Public Key:\n{}", public_key_pem);
    }

    fn encrypt_bidder_amount(amount: &u128, pbk: &RsaPublicKey) -> Vec<u8> {
        let data = amount.to_string();
        pbk.encrypt(&mut OsRng, Pkcs1v15Encrypt, data.as_bytes())
            .expect("failed to encrypt bidder data")
    }
}
