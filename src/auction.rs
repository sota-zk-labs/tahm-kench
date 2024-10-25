use std::fs;
use aligned_sdk::core::types::{Network, PriceEstimate, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{estimate_fee, get_next_nonce, submit_and_wait_verification};
use aligned_sp1_prover::AuctionData;
use anyhow::Result;
use ecies::PublicKey;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};
use sp1_sdk::{ProverClient, SP1Stdin};

const ELF: &[u8] = include_bytes!("../sp1-prover/elf/riscv32im-succinct-zkvm-elf");
const ENCRYPTION_PUBLIC_KEY: &str = include_str!("../sp1-prover/pbk");

abigen!(zkAuction, "./src/assets/zk_auction.json");

// pub async fn approve_nft(
//     signer: SignerMiddleware<Provider<Http>, LocalWallet>,
//     contract_address: &Address,
//     nft_contract_address: &Address,
//     token_id: U256
// ) -> Result<()> {
//     let auction_contract = zkAuction::new(*contract_address, signer.into());
//     let nft_contract = Contract::new(nft_contract_address, ERC721_ABI.clone(), signer.clone());
//
//
//
//     println!("Transaction successful: {:?}", receipt);
//     Ok(())
//
// }

pub async fn create_new_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    public_key_hex: Bytes,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let contract_caller = contract.create_auction(
        public_key_hex,
        nft_contract_address,
        token_id,
        name,
        description,
        target_price,
        duration,
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create auction successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn get_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    auction_id: U256,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let auction = contract.auctions(auction_id).call().await?;
    let (seller, pk, asset, item, winner, deposit_price, end_time, ended) = auction;
    println!("Auction Details:");
    println!("Name: {}", asset.name);
    println!("Seller: {:?}", seller);
    println!("Seller's public key: {:?}", pk);
    println!("Description: {}", asset.description);
    println!("Item:");
    println!("  Address of NFT Contract: {:?}", item.nft_contract);
    println!("  Token ID: {:?}", item.token_id);
    println!("Winner:");
    println!("  Address: {:?}", winner.winner);
    println!("  Encrypted Price: {:?}", winner.price);
    println!("Deposit price: {:?} USDT", deposit_price.low_u128());
    println!("End Time: {:?}", end_time.low_u128());
    println!("Ended: {}", ended);
    Ok(())
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(())
}

// pub fn encrypt_price(bid_price: U256) -> Bytes {
//
// }
pub async fn approve_deposit() {}
pub async fn create_bid(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    auction_id: U256,
    bid_price: U256,
) -> Result<()> {
    let covert_price: [u8; 32] = bid_price.into();
    let covert_price_hex = hex::encode(covert_price);
    let covert_price_bytes: Bytes = hex::decode(&covert_price_hex)
        .expect("Failed to decode hex string") // Handle potential errors
        .into(); // Convert Vec<u8> to Bytes
    let contract = zkAuction::new(*contract_address, signer.into());
    let contract_caller = contract.new_bid(auction_id, covert_price_bytes);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create bid successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn list_bid(auction_id: U256) {}
//
// pub async fn submit_winner() {}
//
// pub async fn withdraw() {}

/// Return winner and proof for the function `revealWinner` in the contract
pub async fn get_winner_and_submit_proof(
    wallet: Wallet<SigningKey>,
    auction_data: &AuctionData,
) -> Result<(Address, Vec<u8>)> {
    let rpc_url = "https://ethereum-holesky-rpc.publicnode.com";
    let network = Network::Holesky;
    let batcher_url = "wss://batcher.alignedlayer.com";

    let mut stdin = SP1Stdin::new();
    stdin.write(auction_data);
    
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    
    let mut proof = client.prove(&pk, stdin).run()?;
    client.verify(&proof, &vk)?;
    
    let mut pub_input = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner = proof.public_values.read::<Vec<u8>>();
    pub_input.extend(winner.clone()); // winner
    
    let proof = bincode::serialize(&proof).expect("Failed to serialize proof");

    // let proof = include_bytes!("../proof").to_vec();
    // let pub_input = include_bytes!("../pub_input").to_vec();
    // let winner = pub_input[pub_input.len()-20..].to_vec();

    fs::write("proof", &proof).expect("Failed to write proof to file");
    let verification_data = VerificationData {
        proving_system: ProvingSystemId::SP1,
        proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(ELF.to_vec()),
        verification_key: None,
        pub_input: Some(pub_input.clone()),
    };
    let max_fee = estimate_fee(rpc_url, PriceEstimate::Instant)
        .await
        .expect("failed to fetch gas price from the blockchain");
    
    // let max_fee_string = ethers::utils::format_units(max_fee, 18)?;

    // if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
    //     .with_prompt(format!("Aligned will use at most {max_fee_string} eth to verify your proof. Do you want to continue?"))
    //     .interact()
    //     .expect("Failed to read user input") {
    //     return Err(anyhow!(""))
    // }

    let nonce = get_next_nonce(rpc_url, wallet.address(), network)
        .await
        .expect("Failed to get next nonce");

    println!("Submitting your proof...");

    let aligned_verification_data = submit_and_wait_verification(
        batcher_url,
        rpc_url,
        network,
        &verification_data,
        max_fee,
        wallet,
        nonce,
    )
    .await
    .unwrap();

    println!(
        "Proof submitted and verified successfully on batch {}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );

    let mut index_in_batch = [0; 32];
    U256::from(aligned_verification_data.index_in_batch).to_big_endian(&mut index_in_batch);

    let merkle_path =
        aligned_verification_data
            .batch_inclusion_proof
            .merkle_path
            .as_slice()
            .iter()
            .flatten();

    let mut verified_proof = pub_input;
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proof_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .pub_input_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proving_system_aux_data_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proof_generator_addr,
    );
    verified_proof.extend(aligned_verification_data.batch_merkle_root);
    verified_proof.extend(merkle_path);
    verified_proof.extend(index_in_batch);

    Ok((Address::from_slice(&winner), verified_proof))
}

pub fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
    ecies::encrypt(&pbk.serialize(), &amount.to_be_bytes())
        .expect("failed to encrypt bidder data")
}
#[cfg(test)]
mod tests {
    use crate::auction::{encrypt_bidder_amount, ENCRYPTION_PUBLIC_KEY};
    use aligned_sp1_prover::{AuctionData, Bidder};
    use ecies::PublicKey;
    use ethers::prelude::Signer;
    use ethers::signers::LocalWallet;
    use std::{env, fs};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_submit_proof() {
        let wallet = LocalWallet::from_str(&env::var("PRIVATE_KEY").unwrap()).unwrap().with_chain_id(17000u64);
        let pbk = PublicKey::parse((*hex::decode(ENCRYPTION_PUBLIC_KEY).unwrap()).try_into().unwrap()).unwrap();

        let data = AuctionData {
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
        };

        let (winner, verified_proof) = super::get_winner_and_submit_proof(wallet, &data).await.unwrap();
        dbg!(winner);
        fs::write("verified_proof", &verified_proof).unwrap();
    }
}