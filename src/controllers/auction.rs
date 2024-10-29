use std::sync::Arc;

use aligned_sdk::core::types::Network;
use aligned_sp1_prover::{AuctionData, Bidder};
use anyhow::{Context, Result};
use ecies::PublicKey;
use ethers::abi::AbiDecode;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::prelude::{LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};
use ethers::utils::keccak256;
use prover_sdk::{encrypt_bidder_amount, get_winner_and_submit_proof};

use crate::entities::auction::AuctionEntity;

abigen!(nftContract, "./assets/erc721.json");
abigen!(erc20Contract, "./assets/erc20.json");
abigen!(zkAuctionContract, "./assets/ZkAuction.json");

pub async fn create_new_auction(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    auction_contract_address: Address,
    pbk_encryption: &PublicKey,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<()> {
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
    let zk_auction_contract =
        zkAuctionContract::new(auction_contract_address, signer.clone().into());
    let contract_caller = zk_auction_contract.create_auction(
        Bytes::from(pbk_encryption.serialize()),
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
    for log in events {
        if log.topics[0] == H256::from(keccak256("AuctionCreated(uint256,address)")) {
            println!("Create auction successfully with:");
            println!("Owner: {:?}", Address::from(log.topics[2]));
            println!("Auction ID: {:?}", U256::decode(log.topics[1])?);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(())
}

pub async fn get_auction(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<AuctionEntity> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let auction = contract.auctions(auction_id).call().await?;
    let auction_entity = AuctionEntity::from(auction);
    auction_entity.print_info();
    Ok(auction_entity)
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    auction_contract_address: Address,
) -> Result<U256> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(total)
}

pub async fn create_bid(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    auction_contract_address: Address,
    token_address: Address,
    auction_id: U256,
    bid_price: u128,
) -> Result<()> {
    let auction = get_auction(signer.clone(), auction_contract_address, auction_id).await?;
    if U256::from(bid_price) > auction.deposit_price {
        println!("You need bid with price < deposit price");
        return Ok(());
    }
    // Approve token
    let erc20_contract = erc20Contract::new(token_address, signer.clone().into());

    let erc20_contract_caller =
        erc20_contract.approve(auction_contract_address, auction.deposit_price);
    let approve_tx = erc20_contract_caller.send().await?;
    let approve_receipt = approve_tx.await?.unwrap();
    println!("==========================================================================");
    println!("Approve {} token successfully with:", auction.deposit_price);
    println!("Auction ID: {:?}", auction_id);
    println!("Tx: {:?}", approve_receipt.transaction_hash);

    let encryption_key = PublicKey::parse((*auction.encryption_key.to_vec()).try_into()?)
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
        if log.topics[0] == H256::from(keccak256("NewBid(uint256,address,bytes)")) {
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

pub async fn get_list_bids(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
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
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
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
        if log.topics[0] == H256::from(keccak256("AuctionEnded(uint256,address,uint128)")) {
            println!("Reveal winner successfully with:");
            println!("Winner address: {:?}", Address::from(log.topics[2]));
            println!("Auction ID: {:?}", U256::decode(log.topics[1])?);
            println!("Price: {:?}", U128::decode(log.topics[3])?);
            println!("Block: {:?}", log.block_number.unwrap());
            println!("Tx: {:?}", log.transaction_hash.unwrap());
        }
    }
    Ok(())
}

pub async fn withdraw(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
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
