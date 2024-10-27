use std::str::FromStr;

use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use home::home_dir;
use prover_sdk::get_encryption_key;

use crate::config::Config;
use crate::controllers::auction::{
    create_bid, create_new_auction, erc20Contract, nftContract, zkAuctionContract,
};
use crate::entities::auction::AuctionEntity;

#[tokio::test]
async fn test_auction_service() {
    let config_path = "config.toml".to_string();
    let keystore_path = ".zk_auction/keystores/wallet_zk_auction".to_string();

    let config =
        Config::new(&config_path).expect(&format!("Failed to load config from {:?}", &config_path));

    let rpc_url = &config.chain.rpc_url;
    let provider =
        Provider::<Http>::try_from(rpc_url.as_str()).expect("Failed to connect to provider");
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

    let encryption_key = get_encryption_key().unwrap();
    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());
    let wallet_address = signer.clone().address();

    // Set up phase
    // Set up total
    let contract = zkAuctionContract::new(config.contract_address, signer.clone().into());
    let auction_total = contract.auction_count().call().await?;
    // Set up nft
    let nft_contract_address_input = "0xcb356f9df6aff96f8e75054dbfb1fd60ec9d7b73";
    let nft_contract_address = Address::from_str(&nft_contract_address_input).unwrap();

    let new_token_id = auction_total + 1;
    let erc721_contract = nftContract::new(nft_contract_address, signer.clone().into());
    let nft_contract_caller = erc721_contract.mint(wallet_address, new_token_id);
    let nft_tx = nft_contract_caller.send().await?;
    let nft_receipt = nft_tx.await?.unwrap();
    println!(
        "Mint nft successfully with token_id = {} and transaction hash: {:?}",
        new_token_id, nft_receipt.transaction_hash
    );

    // Set up token
    let token_mint = U256::from(10000000000000000000u128);
    let token_contract = erc20Contract::new(config.token_address, signer.clone().into());
    let token_contract_caller = token_contract.mint(wallet_address, token_mint);
    let token_tx = token_contract_caller.send().await?;
    let token_receipt = token_tx.await?.unwrap();
    println!(
        "Mint token successfully transaction hash: {:?}",
        token_receipt.transaction_hash
    );

    // Create new auction
    let name = "test".to_string();
    let description = "nothing".to_string();

    let _ = create_new_auction(
        signer.clone(),
        config.contract_address,
        &encryption_key,
        name,
        description,
        nft_contract_address,
        new_token_id,
        token_mint,
        U256::from(3600),
    )
        .await
        .unwrap();

    // Create new bid
    let _ = create_bid(
        signer.clone(),
        config.contract_address,
        config.token_address,
        auction_total,
        1,
    )
        .await
        .unwrap();
}
