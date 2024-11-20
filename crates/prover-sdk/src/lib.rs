#![feature(duration_constructors)]
extern crate core;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{env, fs};

use aligned_sp1_prover::AuctionData;
use anyhow::Result;
use ecies::{PublicKey, SecretKey};
use ethers::types::{Address, Bytes};
use sp1_sdk::{HashableKey, Prover, ProverClient, SP1Stdin};


/// Return winner and proof for the function `revealWinner` in the contract
/// 
/// # Arguments 
/// 
/// * `auction_data`: The auction data containing the bidders and their encrypted amounts
/// 
/// returns: Result<(H160, u128, Bytes, Bytes), Error> (winner address, winner amount, public values, proof)
pub async fn find_winner(auction_data: &AuctionData) -> Result<(Address, u128, Bytes, Bytes)> {
    println!("Creating proof...");

    let mut stdin = SP1Stdin::new();
    stdin.write(auction_data);
    stdin.write(&get_private_encryption_key()?.serialize().to_vec());

    let client = ProverClient::new();

    let elf = &get_elf()?;
    let (pk, vk) = client.setup(elf);
    let mut proof = client.prove(&pk, stdin).plonk().run()?;

    println!("Proof created successfully");

    client.verify(&proof, &vk)?;

    let _hash_data = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = Address::from_slice(proof.public_values.read::<Vec<u8>>().as_slice()); // winner address
    let winner_amount = proof.public_values.read::<u128>(); // winner amount

    fs::write("public_values", hex::encode(proof.public_values.as_slice()))?;
    fs::write("proof", hex::encode(proof.bytes()))?;
    fs::write("verifying_key", vk.bytes32())?;
    Ok((
        winner_addr,
        winner_amount,
        Bytes::from(proof.public_values.to_vec()),
        Bytes::from(proof.bytes()),
    ))
}

/// Encrypts the amount of a bidder using the public key of the owner
///
/// # Arguments
///
/// * `amount`: bid amount
/// * `pbk`: public key of the owner
///
/// returns: Vec<u8, Global> encrypted amount
pub fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
    ecies::encrypt(&pbk.serialize(), &amount.to_be_bytes()).expect("failed to encrypt bidder data")
}

/// Get the public encryption key of the owner
pub fn get_encryption_key() -> Result<PublicKey> {
    Ok(PublicKey::parse(
        &hex::decode(fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/encryption_key"),
        )?)?
        .try_into()
        .unwrap(),
    )
    .expect("parsing public encryption key failed"))
}

/// Get the private encryption key of the owner
pub fn get_private_encryption_key() -> Result<SecretKey> {
    Ok(SecretKey::parse_slice(&hex::decode(fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/private_encryption_key"),
    )?)?)
    .expect("parsing private encryption key failed"))
}

/// Get the ELF file that was compiled with the SP1 prover
pub fn get_elf() -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    File::open(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../sp1-prover//elf/riscv32im-succinct-zkvm-elf"),
    )?
    .read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Flatten a 2D array into a 1D array
///
/// # Arguments
///
/// * `vec`: 2D array
///
/// returns: Vec<u8, Global> Flatten array
pub fn flatten(vec: &[[u8; 32]]) -> Vec<u8> {
    let mut res = vec![];
    for v in vec.iter() {
        res.extend_from_slice(v);
    }
    res
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::{env, fs};

    use aligned_sp1_prover::{AuctionData, Bidder};
    use ethers::abi::Address;
    use ethers::contract::abigen;
    use ethers::middleware::SignerMiddleware;
    use ethers::prelude::{Http, LocalWallet, Provider};
    use ethers::signers::Signer;
    use ethers::types::{Bytes, H160};
    use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};

    use crate::{encrypt_bidder_amount, get_elf, get_encryption_key, get_private_encryption_key};

    #[tokio::test]
    async fn test_submit_proof() {
        let (_, winner_amount, _, _) = super::find_winner(&auction_data()).await.unwrap();
        dbg!(winner_amount);
    }

    // Plonk prover with external `ecies` crates: 2 bidders, 17 mins, proof 868 bytes
    #[tokio::test]
    async fn test_sp1_prover() {
        // find_winner(&auction_data(), PrivateKey::from_bytes(hex::decode(ENCRYPTION_PRIVATE_KEY).unwrap()));
        let elf = get_elf().unwrap();

        let mut stdin = SP1Stdin::new();
        stdin.write(&auction_data());
        stdin.write(&get_private_encryption_key().unwrap().serialize().to_vec());

        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf.as_slice());

        println!("Generating proof...");
        let Ok(mut proof) = client.prove(&pk, stdin).plonk().run() else {
            println!("Something went wrong!");
            return;
        };

        println!("Proof generated successfully. Verifying proof off-chain...");
        client.verify(&proof, &vk).expect("verification failed");
        println!("Proof was verified successfully.");

        // write data to file
        fs::write("public_values", hex::encode(proof.public_values.as_slice())).unwrap();
        fs::write("proof", hex::encode(proof.bytes())).unwrap();
        fs::write("verifying_key", vk.bytes32()).unwrap();

        let hash_data = proof.public_values.read::<[u8; 32]>();
        println!("{:?}", hash_data);
        let winner_addr = proof.public_values.read::<Vec<u8>>();
        println!("{:?}", winner_addr);
        println!("Proof length: {} bytes", proof.bytes().len());
    }

    #[tokio::test]
    async fn verify_proof_onchain() {
        // verify proof on-chain
        abigen!(
            sp1Verifier,
            r#"[
                function verifyProof(bytes32 programVKey, bytes calldata publicValues, bytes calldata proofBytes) external view returns ()
            ]"#
        );
        let wallet = LocalWallet::from_str(&env::var("PRIVATE_KEY").unwrap())
            .unwrap()
            .with_chain_id(17000u64);
        let signer = SignerMiddleware::new(
            Arc::new(
                Provider::<Http>::try_from("https://ethereum-holesky-rpc.publicnode.com").unwrap(),
            ),
            wallet.clone(),
        );
        let contract = sp1Verifier::new(
            Address::from_str("0x3B6041173B80E77f038f3F2C0f9744f04837185e").unwrap(),
            signer.into(),
        );
        let contract_caller = contract.verify_proof(
            hex::decode(
                fs::read_to_string("verifying_key")
                    .unwrap()
                    .strip_prefix("0x")
                    .unwrap(),
            )
            .unwrap()
            .try_into()
            .unwrap(),
            Bytes::from_str(&fs::read_to_string("public_values").unwrap()).unwrap(),
            Bytes::from_str(&fs::read_to_string("proof").unwrap()).unwrap(),
        );
        println!("Verifying proof on-chain...");
        let tx = contract_caller.send().await.unwrap();
        let receipt = tx.await.unwrap().unwrap();
        let tx_hash = receipt.transaction_hash;
        println!("Verified proof on-chain. Transaction hash: {:?}", tx_hash);
    }

    #[test]
    fn test_type() {
        let x = H160::from_str("0xeDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap();
        assert_eq!(
            hex::encode(x.as_fixed_bytes()),
            "ede4c2b4bdbe580750a99f016b0a1581c3808fa3".to_string()
        );

        let y = Bytes::from(vec![1, 2, 3]);
        assert_eq!(y.to_vec(), vec![1, 2, 3]);
    }

    fn auction_data() -> AuctionData {
        let pbk = get_encryption_key().unwrap();

        AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&3, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&2, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
            ],
            id: vec![0; 32],
        }
    }
}
