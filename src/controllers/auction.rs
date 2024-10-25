use std::fs;

use aligned_sdk::core::types::{Network, PriceEstimate, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{estimate_fee, get_next_nonce, submit_and_wait_verification};
use aligned_sp1_prover::{AuctionData, Bidder};
use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use ecies::PublicKey;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};
use sp1_sdk::{ProverClient, SP1Stdin};

use crate::entities::auction::{AssetEntity, AuctionEntity, WinnerEntity};

const ELF: &[u8] = include_bytes!("../../sp1-prover/elf/riscv32im-succinct-zkvm-elf");
const ENCRYPTION_PUBLIC_KEY: &str = include_str!("../../sp1-prover/pbk");

abigen!(erc721Contract, "./assets/erc721.json");
abigen!(erc20Contract, "./assets/erc20.json");
abigen!(zkAuctionContract, "./assets/ZkAuction.json");

pub async fn create_new_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    public_key_hex: Bytes,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<()> {
    // Approve NFT
    let erc721_contract = erc721Contract::new(nft_contract_address, signer.clone().into());
    let erc721_contract_caller = erc721_contract.approve(auction_contract_address, token_id);
    let approve_tx = erc721_contract_caller.send().await?;
    let _ = approve_tx.await?.unwrap();

    // Create Auction
    let zk_auction_contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = zk_auction_contract.create_auction(
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
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<AuctionEntity> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let auction = contract.auctions(auction_id).call().await?;

    let (owner_, owner_public_key_, asset_, winner_, deposit_price_, end_time_, ended_) = auction;
    let convert_auction = AuctionEntity {
        owner: owner_,
        owner_public_key: owner_public_key_,
        asset: AssetEntity {
            name: asset_.name,
            description: asset_.description,
            nft_contract_address: asset_.nft_contract,
            token_id: asset_.token_id,
        },
        winner: WinnerEntity {
            winner_address: winner_.winner,
            price: winner_.price,
        },
        deposit_price: deposit_price_,
        end_time: end_time_,
        ended: ended_,
    };
    let _ = convert_auction.print_info();
    Ok(convert_auction)
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
) -> Result<()> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(())
}

// pub fn encrypt_price(bid_price: U256) -> Bytes {
//
// }

pub async fn create_bid(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    token_address: Address,
    auction_id: U256,
    bid_price: U256,
) -> Result<()> {
    let auction = get_auction(signer.clone(), auction_contract_address, auction_id).await?;
    // Approve token
    let erc20_contract = erc20Contract::new(token_address, signer.clone().into());
    let erc20_contract_caller =
        erc20_contract.approve(auction_contract_address, auction.asset.token_id);
    let approve_tx = erc20_contract_caller.send().await?;
    let _ = approve_tx.await?.unwrap();

    // Fake encrypted price
    let covert_price: [u8; 32] = bid_price.into();
    let covert_price_hex = hex::encode(covert_price);
    let covert_price_bytes: Bytes = hex::decode(&covert_price_hex)
        .expect("Failed to decode hex string") // Handle potential errors
        .into(); // Convert Vec<u8> to Bytes

    // Create bid
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.place_bid(auction_id, covert_price_bytes);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create bid successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn get_list_bids(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<Vec<Bidder>> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let bids = contract.get_bids(auction_id).call().await?;
    let list_bids: Vec<_> = bids
        .into_iter()
        .map(|element| Bidder {
            encrypted_amount: element.encrypted_price.to_vec(),
            address: element.bidder.as_fixed_bytes().to_vec(),
        })
        .collect();
    Ok(list_bids)
}

pub async fn reveal_winner(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
    wallet: Wallet<SigningKey>,
    rpc_url: &str,
    network: Network,
    batcher_url: &str
) -> Result<()> {
    // Get list bids
    let bidders = get_list_bids(signer.clone(), auction_contract_address, auction_id)
        .await
        .context(format!(
            "Failed to get list bids from auction with id: {}",
            auction_id
        ))?;
    //Send to SP1
    let mut auc_id = [0; 32];
    auction_id.to_big_endian(&mut auc_id);
    let (winner_addr, winner_amount, verified_proof) = get_winner_and_submit_proof(
        wallet,
        &AuctionData {
            bidders,
            id: auc_id.to_vec(),
        },
        rpc_url,
        network,
        batcher_url
    )
    .await?;

    // // Submit proof to SMC
    // let winner = Winner{
    //     winner_address: Default::default(),
    // //     price: Default::default()
    // };

    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.finalize_auction(
        auction_id,
        Winner {
            winner: winner_addr,
            price: U256::from(winner_amount),
        },
        Bytes::from(verified_proof),
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Reveal winner successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn withdraw(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<()> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.withdraw(auction_id);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Withdraw deposit successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

/// Return winner and proof for the function `revealWinner` in the contract
pub async fn get_winner_and_submit_proof(
    wallet: Wallet<SigningKey>,
    auction_data: &AuctionData,
    rpc_url: &str,
    network: Network,
    batcher_url: &str
) -> Result<(Address, u128, Vec<u8>)> {

    let mut stdin = SP1Stdin::new();
    stdin.write(auction_data);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    let mut proof = client.prove(&pk, stdin).run()?;
    client.verify(&proof, &vk)?;

    let mut pub_input = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = proof.public_values.read::<Vec<u8>>();
    let winner_amount = proof.public_values.read::<u128>();
    pub_input.extend(winner_addr.clone()); // winner

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

    #[cfg(not(test))]
    let max_fee_string = ethers::utils::format_units(max_fee, 18)?;

    #[cfg(not(test))]
    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Aligned will use at most {max_fee_string} eth to verify your proof. Do you want to continue?"))
        .interact()
        .expect("Failed to read user input") {
        return Err(anyhow!(""))
    }

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

    let merkle_path = aligned_verification_data
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

    Ok((
        Address::from_slice(&winner_addr),
        winner_amount,
        verified_proof,
    ))
}

pub fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
    ecies::encrypt(&pbk.serialize(), &amount.to_be_bytes()).expect("failed to encrypt bidder data")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::{env, fs};
    use aligned_sdk::core::types::Network;
    use aligned_sp1_prover::{AuctionData, Bidder};
    use ecies::PublicKey;
    use ethers::prelude::Signer;
    use ethers::signers::LocalWallet;
    use ethers::types::{Bytes, H160};

    use crate::controllers::auction::{encrypt_bidder_amount, ENCRYPTION_PUBLIC_KEY};

    #[tokio::test]
    async fn test_submit_proof() {
        let rpc_url = "https://ethereum-holesky-rpc.publicnode.com";
        let network = Network::Holesky;
        let batcher_url = "wss://batcher.alignedlayer.com";
        let wallet = LocalWallet::from_str(&env::var("PRIVATE_KEY").unwrap())
            .unwrap()
            .with_chain_id(17000u64);
        let pbk = PublicKey::parse(
            (*hex::decode(ENCRYPTION_PUBLIC_KEY).unwrap())
                .try_into()
                .unwrap(),
        )
        .unwrap();

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

        let (_winner_addr, winner_amount, verified_proof) =
            super::get_winner_and_submit_proof(wallet, &data, rpc_url, network, batcher_url)
                .await
                .unwrap();
        dbg!(winner_amount);
        fs::write("verified_proof", &verified_proof).unwrap();
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
}
