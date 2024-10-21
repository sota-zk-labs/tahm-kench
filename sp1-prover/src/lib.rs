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
    pub encrypted_data: Vec<u8>,
    pub address: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BidderDecryptedData {
    pub amount: u128,
    pub timestamp: u64,
}

impl From<String> for BidderDecryptedData {
    fn from(value: String) -> Self {
        let data = value.split(',').collect::<Vec<&str>>();
        BidderDecryptedData {
            amount: data[0].parse().unwrap(),
            timestamp: data[1].parse().unwrap(),
        }
    }
}

impl From<&BidderDecryptedData> for String {
    fn from(value: &BidderDecryptedData) -> Self {
        format!("{},{}", value.amount, value.timestamp)
    }
}

pub fn decrypt_bidder_data(pvk: &RsaPrivateKey, bidder: &Bidder) -> BidderDecryptedData {
    let data = String::from_utf8(
        pvk.decrypt(Pkcs1v15Encrypt, &bidder.encrypted_data)
            .expect("failed to decrypt"),
    )
    .unwrap();
    data.into()
}

#[cfg(test)]
mod tests {
    use crate::{AuctionData, Bidder, BidderDecryptedData, PVK_PEM};
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
                    encrypted_data: encrypt_bidder_data(
                        &BidderDecryptedData {
                            amount: 2,
                            timestamp: 10,
                        },
                        &pbk,
                    ),
                    address: vec![5; 32],
                },
                Bidder {
                    encrypted_data: encrypt_bidder_data(
                        &BidderDecryptedData {
                            amount: 1,
                            timestamp: 5,
                        },
                        &pbk,
                    ),
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
            encrypted_data: encrypt_bidder_data(
                &BidderDecryptedData {
                    amount: 1e23 as u128,
                    timestamp: 1e10 as u64,
                },
                &pbk,
            ),
            address: vec![0; 32],
        };
        let decrypted_data = super::decrypt_bidder_data(&pvk, &bidder);
        assert_eq!(decrypted_data.timestamp, 1e10 as u64);
    }

    #[test]
    fn test_gen_key() {
        // Generate a 2048-bit RSA private key
        let mut rng = OsRng;
        let bit_size = env::var("BIT_SIZE").unwrap_or("361".to_string()).parse().unwrap();
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

    fn encrypt_bidder_data(data: &BidderDecryptedData, pbk: &RsaPublicKey) -> Vec<u8> {
        let data = String::from(data);
        pbk.encrypt(&mut OsRng, Pkcs1v15Encrypt, data.as_bytes())
            .expect("failed to encrypt bidder data")
    }
}
