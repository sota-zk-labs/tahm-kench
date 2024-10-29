use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use aligned_sdk::core::types::Network;
use chrono::Utc;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::Provider;
use ethers::signers::{LocalWallet, Signer};
use home::home_dir;
use prover_sdk::get_encryption_key;

use crate::config::Config;
use crate::controllers::auction::{
    create_bid, create_new_auction, erc20Contract, nftContract, reveal_winner, zkAuctionContract,
};

#[tokio::test]
async fn test_auction_service() {
    let config_path = "config.toml".to_string();
    let config =
        Config::new(&config_path).expect(&format!("Failed to load config from {:?}", &config_path));

    // let (owner_signer, owner_wallet_address, wallet) = set_up_wallet(config.clone(), ".foundry/keystores/owner1".to_string()).await;
    // let (bidder_1_signer, bidder_1_wallet_address, wallet) = set_up_wallet(config.clone(), ".foundry/keystores/bider1".to_string()).await;

    let (signer, wallet_address, wallet) =
        set_up_wallet(config.clone(), ".foundry/keystores/dung1".to_string()).await;

    let rpc_url = &config.chain.rpc_url;
    let network = Network::from_str(&config.chain.network).unwrap();
    let aligned_batcher_url = &config.chain.aligned_batcher_url;

    let contract = zkAuctionContract::new(config.contract_address, signer.clone().into());
    let auction_total = contract.auction_count().call().await.unwrap();

    let nft_contract_address_input = "0x4dab910a86affd90d888d82f16e9faaf33f5ee61";
    let nft_contract_address = Address::from_str(&nft_contract_address_input).unwrap();

    let new_token_id = auction_total + 1;
    let token_mint = U256::from(1000u128);
    println!("New token ID: {}", new_token_id);
    // let owner_address = Address::from_str("0xeDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap();
    // let _ = set_up_nft(
    //     signer.clone(),
    //     owner_address.clone(),
    //     nft_contract_address,
    //     new_token_id,
    // )
    // .await;
    let _ = set_up_nft(
        signer.clone(),
        wallet_address.clone(),
        nft_contract_address,
        new_token_id,
    )
    .await;

    // let bidder_address = Address::from_str("0xB3bD6356709809786C1dA9B777732774215E5cB6").unwrap();
    // let _ = set_up_token(config.clone(), signer.clone(), bidder_address, token_mint).await;
    let _ = set_up_token(config.clone(), signer.clone(), wallet_address.clone(), token_mint).await;

    // Test create new auction success
    let name = "test".to_string();
    let description = "nothing".to_string();
    let encryption_key = get_encryption_key().unwrap();

    // Create new auction
    let _ = create_new_auction(
        signer.clone(),
        config.contract_address,
        &encryption_key,
        name,
        description,
        nft_contract_address,
        new_token_id,
        token_mint,
        U256::from(60),
    )
    .await
    .unwrap();

    // Create new bid
    let _ = create_bid(
        signer.clone(),
        config.contract_address,
        config.token_address,
        new_token_id,
        900,
    )
    .await
    .unwrap();

    // println!("Sleep 60 second...");
    // sleep(Duration::from_secs(60));
    // println!("Sleep over");
    // println!("Utc now: {:?}", Utc::now());

    // // Get list bid
    // let _ = reveal_winner(
    //     signer.clone(),
    //     config.contract_address,
    //     new_token_id,
    //     wallet,
    //     rpc_url,
    //     network,
    //     aligned_batcher_url,
    // )
    // .await
    // .unwrap();
}

async fn set_up_token(
    config: Config,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    wallet_address: Address,
    token_mint: U256,
) {
    // Set up token
    let token_contract = erc20Contract::new(config.token_address, signer.clone().into());
    let token_contract_caller = token_contract.mint(wallet_address, token_mint);
    let token_tx = token_contract_caller.send().await.unwrap();
    let token_receipt = token_tx.await.unwrap().unwrap();
    println!("==========================================================================");
    println!("Mint {} token successfully with:", token_mint);
    println!("Tx: {:?}", token_receipt.transaction_hash);
}

async fn set_up_nft(
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    wallet_address: Address,
    nft_contract_address: Address,
    new_token_id: U256,
) {
    // Set up nft
    let erc721_contract = nftContract::new(nft_contract_address, signer.clone().into());
    let nft_contract_caller = erc721_contract.mint(wallet_address, new_token_id);
    let nft_tx = nft_contract_caller.send().await.unwrap();
    let nft_receipt = nft_tx.await.unwrap().unwrap();
    println!("==========================================================================");
    println!("Mint NFT successfully with:",);
    println!("Token ID: {:?}", new_token_id);
    println!("Tx: {:?}", nft_receipt.transaction_hash);
}

async fn set_up_wallet(
    config: Config,
    keystore_path: String,
) -> (
    SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    Address,
    Wallet<SigningKey>,
) {
    let rpc_url = config.chain.rpc_url.as_str();
    let provider = Provider::<Http>::try_from(rpc_url).expect("Failed to connect to provider");
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to get chain_id");

    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");

    let home_dir = home_dir().expect("Failed to get home directory");
    let path = home_dir.join(&keystore_path);
    let wallet = LocalWallet::decrypt_keystore(path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    let signer = SignerMiddleware::new(Arc::new(provider.clone()), wallet.clone());
    let wallet_address = signer.clone().address();

    (signer, wallet_address, wallet)
}
