#![feature(duration_constructors)]
extern crate core;

use anyhow::Result;
use auction_sp1_prover::AuctionData;
use ecies::private_key::PrivateKey;
use ecies::public_key::PublicKey;
use ecies::symmetric_encryption::simple::SimpleSE;
use ecies::Ecies;
use ethers::core::rand::rngs::OsRng;
use ethers::types::{Address, Bytes};
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use std::path::PathBuf;
use std::{env, fs};

const ELF: &[u8] = include_bytes!("../../sp1-prover/elf/riscv32im-succinct-zkvm-elf");

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
    stdin.write(&get_private_encryption_key()?.to_bytes().to_vec());
    
    let client = ProverClient::new();
    
    let (pk, vk) = client.setup(ELF);
    println!("Verifying key: {}", vk.bytes32());
    let mut proof = client.prove(&pk, stdin).plonk().run()?;
    println!("Proof created successfully");
    
    client.verify(&proof, &vk)?;
    
    let proof_bytes = proof.bytes();
    fs::write("public_values", hex::encode(proof.public_values.as_slice()))?;
    fs::write("proof", hex::encode(&proof_bytes))?;
    fs::write("verifying_key", vk.bytes32())?;
    
    let _hash_data = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = Address::from_slice(proof.public_values.read::<Vec<u8>>().as_slice()); // winner address
    let winner_amount = proof.public_values.read::<u128>(); // winner amount
    Ok((
        winner_addr,
        winner_amount,
        Bytes::from(proof.public_values.to_vec()),
        Bytes::from(proof_bytes),
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
    let scheme = Ecies::<SimpleSE>::from_pvk(PrivateKey::from_rng(&mut OsRng));
    scheme.encrypt(&mut OsRng, pbk, &amount.to_be_bytes())
}

/// Get the public encryption key of the owner
pub fn get_encryption_key() -> Result<PublicKey> {
    Ok(PublicKey::from_bytes(&hex::decode(fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/encryption_key"),
    )?)?))
}

/// Get the private encryption key of the owner
pub fn get_private_encryption_key() -> Result<PrivateKey> {
    Ok(PrivateKey::from_bytes(&hex::decode(fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/private_encryption_key"),
    )?)?))
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
    use std::str::FromStr;
    use std::sync::Arc;
    use std::{env, fs};

    use auction_sp1_prover::{AuctionData, Bidder};
    use ethers::abi::Address;
    use ethers::contract::abigen;
    use ethers::middleware::SignerMiddleware;
    use ethers::prelude::{Http, LocalWallet, Provider};
    use ethers::signers::Signer;
    use ethers::types::Bytes;

    use crate::{encrypt_bidder_amount, get_encryption_key};

    // Plonk prover with external `ecies` crates: 2 bidders, 17 mins, proof 868 bytes
    #[tokio::test]
    async fn test_find_winner() {
        let (_, winner_amount, _, _) = super::find_winner(&auction_data()).await.unwrap();
        dbg!(winner_amount);
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
