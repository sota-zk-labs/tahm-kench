use aligned_sdk::core::types::Network;
use aligned_sp1_prover::{AuctionData, Bidder};
use anyhow::{anyhow, Context, Result};
use chrono::{TimeZone, Utc};
use ecies::PublicKey;
use ethers::abi::AbiDecode;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::types::{Address, Bytes, U256};
use ethers::utils::keccak256;
use prover_sdk::{encrypt_bidder_amount, get_winner_and_submit_proof};

use crate::types::EthSigner;

abigen!(nftContract, "./assets/erc721.json");
abigen!(erc20Contract, "./assets/erc20.json");
abigen!(zkAuctionContract, "./assets/ZkAuction.json");

/// Creates a new auction.
///
/// # Arguments
///
/// * `signer` - A configured `SignerMiddleware` used for signing and sending transactions.
/// * `auction_contract_address` - The contract address of the auction platform where the auction will be created.
/// * `pk_encryption` - The public key used for encrypting auction-specific data.
/// * `token_addr` - An IERC20 token address used for the auction.
/// * `name` - A string containing the name of the auction.
/// * `description` - A string describing the auction.
/// * `nft_contract_address` - The address of the ERC-721 contract managing the NFT to be auctioned.
/// * `token_id` - ID of the NFT token to be auctioned.
/// * `target_price` - The price expected for the auction to be successful.
/// * `duration` - The duration for which the auction will be active, measured in blockchain blocks or seconds, depending on the implementation.
pub async fn create_new_auction(
    signer: EthSigner,
    auction_contract_address: Address,
    pbk_encryption: &PublicKey,
    token_addr: Address,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<U256> {
    // Approve NFT
    let erc721_contract = nftContract::new(nft_contract_address, signer.clone().into());
    let erc721_contract_caller = erc721_contract.approve(auction_contract_address, token_id);
    let approve_tx = erc721_contract_caller.send().await?;
    let approve_receipt = approve_tx.await?.unwrap();
    println!("==========================================================================");
    println!("Approve NFT successfully with:",);
    println!("Token ID: {:?}", token_id);
    println!("Tx: {:?}", approve_receipt.transaction_hash);
    // Create Auction
    let zk_auction_contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = zk_auction_contract.create_auction(
        Bytes::from(pbk_encryption.serialize()),
        token_addr,
        nft_contract_address,
        token_id,
        name,
        description,
        target_price,
        duration,
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let events = receipt.logs;
    println!("==========================================================================");
    let mut auction_id = U256::zero();
    for log in events {
        if log.topics[0] == H256::from(keccak256(b"AuctionCreated(uint256,address)")) {
            println!("Create auction successfully with:");
            println!("Owner: {:?}", Address::from(log.topics[2]));
            auction_id = U256::decode(log.topics[1])?;
            println!("Auction ID: {:?}", auction_id);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(auction_id)
}

/// Fetches details of a specific auction by ID.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
/// * `auction_id` - ID of the auction.
///
/// # Returns
///
/// Returns a tuple containing:
/// - `owner` - The address of the auction owner.
/// - `encryption_key` - The public key used for encrypted bids.
/// - `asset` - Struct holding information about the auctioned item (name, description, NFT contract address, token ID).
/// - `winner` - Struct containing details of the current highest bidder, including address and encrypted price.
/// - `deposit_price` - The minimum deposit price for the auction in USDT.
/// - `end_time` - The auction end timestamp.
/// - `ended` - A boolean indicating whether the auction has ended.
pub async fn get_auction(
    signer: EthSigner,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<(Address, Bytes, Address, Asset, Winner, U256, U256, bool)> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let (owner, encryption_key, token_addr, asset, winner, deposit_price, end_time, ended) =
        contract.auctions(auction_id).call().await?;
    println!("==========================================================================");
    println!("Auction Details:");
    println!("Name: {}", asset.name);
    println!("Seller: {:?}", owner);
    println!("Seller's public encryption key: {:?}", encryption_key);
    println!("Token address: {}", &token_addr);
    println!("Description: {}", asset.description);
    println!("Item:");
    println!("  Address of NFT Contract: {:?}", asset.nft_contract);
    println!("  Token ID: {:?}", asset.token_id);
    println!("Winner:");
    println!("  Address: {:?}", winner.winner);
    println!("  Encrypted Price: {:?}", winner.price);
    println!("Deposit price: {:?} USDT", deposit_price);
    println!(
        "End Time: {:?}",
        Utc.timestamp_opt(end_time.as_u128() as i64, 0).unwrap()
    );
    println!("Ended: {}", ended);
    Ok((
        owner,
        encryption_key,
        token_addr,
        asset,
        winner,
        deposit_price,
        end_time,
        ended,
    ))
}

/// Get the total count of auctions on auction contract.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
///
/// # Returns
///
/// Returns the total number of auctions (`U256`) currently managed by the contract.
pub async fn get_total_auction(
    signer: EthSigner,
    auction_contract_address: Address,
) -> Result<U256> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(total)
}

/// Places a new bid on a specific auction.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
/// * `auction_id` - ID of the auction to bid on.
/// * `bid_price` - The bid price to submit, in `u128`.
///
/// # Returns
///
/// Returns `Ok(())` if the bid is successfully placed or if the bid fails due to an invalid bid price.
///
pub async fn create_bid(
    signer: EthSigner,
    auction_contract_address: Address,
    auction_id: U256,
    bid_price: u128,
) -> Result<()> {
    let (_, encryption_key, token_address, _, _, deposit_price, _, _) =
        get_auction(signer.clone(), auction_contract_address, auction_id).await?;
    if U256::from(bid_price) > deposit_price {
        return Err(anyhow!("You need bid with price < deposit price"));
    }
    // Approve token
    let erc20_contract = erc20Contract::new(token_address, signer.clone().into());

    let erc20_contract_caller = erc20_contract.approve(auction_contract_address, deposit_price);
    let approve_tx = erc20_contract_caller.send().await?;
    let approve_receipt = approve_tx.await?.unwrap();
    println!("==========================================================================");
    println!("Approve {} token successfully with:", deposit_price);
    println!("Auction ID: {:?}", auction_id);
    println!("Tx: {:?}", approve_receipt.transaction_hash);

    let encryption_key = PublicKey::parse((*encryption_key.to_vec()).try_into()?)
        .expect("Wrong on-chain encryption key");
    // Encrypted price
    let encrypted_price = encrypt_bidder_amount(&bid_price, &encryption_key);

    // Create bid
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.place_bid(auction_id, Bytes::from(encrypted_price.clone()));
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let events = receipt.logs;
    println!("==========================================================================");
    for log in events {
        if log.topics[0] == H256::from(keccak256(b"NewBid(uint256,address,bytes)")) {
            println!("Create new bid successfully with:");
            println!("Bidder address: {:?}", Address::from(log.topics[2]));
            println!("Auction ID: {:?}", U256::decode(log.topics[1])?);
            println!("Encrypted price: {:?}", encrypted_price);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(())
}

/// Retrieves a list of bids for a specified auction.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
/// * `auction_id` - ID of the auction.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<Bidder>` with each bid's encrypted amount and bidder's address.
pub async fn get_list_bids(
    signer: EthSigner,
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

/// Reveals the auction winner.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
/// * `auction_id` - ID of the auction.
/// * `wallet` - Wallet used to sign the winner's proof and other operations.
/// * `rpc_url` - URL of the Ethereum node to connect to.
/// * `network` - The network on which the auction is deployed (e.g., Ethereum mainnet or testnet).
/// * `batcher_url` - URL of the batcher service for processing ZKP proofs.
///
/// # Returns
///
/// Returns `Result<()>` indicating success or failure.
///
/// # Workflow
///
/// 1. Retrieves the list of bidders for the specified auction.
/// 2. Calls an external function, `get_winner_and_submit_proof`, which determines the winner and generates a ZKP.
/// 3. Submits the proof and winner information to the smart contract's `finalize_auction` function.
/// 4. Processes transaction logs to verify the result.
pub async fn reveal_winner(
    signer: EthSigner,
    auction_contract_address: Address,
    auction_id: U256,
    wallet: Wallet<SigningKey>,
    rpc_url: &str,
    network: Network,
    batcher_url: &str,
) -> Result<()> {
    // Get list bids
    let bidders = get_list_bids(signer.clone(), auction_contract_address, auction_id)
        .await
        .context(format!(
            "Failed to get list bids from auction with id: {}",
            auction_id
        ))?;
    if bidders.len() == 0 {
        println!("No one bid, refund NFT");
        let _ = refund_nft_to_owner(
            signer.clone(),
            auction_contract_address.clone(),
            auction_id.clone(),
        )
        .await
        .unwrap_or_else(|e| {
            println!("{}", e);
            panic!("Failed to refund nft from auction with id: {}", auction_id);
        });
        return Ok(());
    }
    println!("bidders: {:?}", bidders);
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
        batcher_url,
    )
    .await?;

    // Submit proof to SMC
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.finalize_auction(
        auction_id,
        Winner {
            winner: winner_addr,
            price: winner_amount,
        },
        Bytes::from(verified_proof),
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let events = receipt.logs;
    println!("==========================================================================");
    for log in events {
        if log.topics[0] == H256::from(keccak256(b"AuctionEnded(uint256,address,uint128)")) {
            println!("Reveal winner successfully with:");
            println!("Winner address: {:?}", Address::from(log.topics[2]));
            println!("Auction ID: {:?}", U256::decode(log.topics[1])?);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(())
}

/// Withdraws the deposit for a specific auction, if applicable, and completes the withdrawal process on the contract.
///
/// # Arguments
///
/// * `signer` - A `SignerMiddleware` configured for interacting with the blockchain and signing transactions.
/// * `auction_contract_address` - The contract address of the auction platform.
/// * `auction_id` - ID of the auction.
///
/// # Returns
///
/// Returns `Result<()>` indicating success or failure.
pub async fn withdraw(
    signer: EthSigner,
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

pub async fn refund_nft_to_owner(
    signer: EthSigner,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<()> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.refund_nft(auction_id);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    println!(
        "Refund NFT successfully with transaction_hash : {:?}",
        receipt.transaction_hash
    );
    let events = receipt.logs;
    println!("==========================================================================");
    for log in events {
        if log.topics[0] == H256::from(keccak256(b"AuctionEnded(uint256,address,uint128)")) {
            println!("Auction Ended with:");
            println!("Winner address: {:?}", Address::from(log.topics[2]));
            println!("Auction ID: {:?}", U256::decode(log.topics[1])?);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(())
}
