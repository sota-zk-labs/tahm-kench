#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;

    use aligned_sdk::core::types::Network;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::rand::rngs::OsRng;
    use ethers::core::rand::RngCore;
    use ethers::prelude::*;
    use ethers::providers::Provider;
    use ethers::signers::{LocalWallet, Signer};
    use home::home_dir;
    use prover_sdk::get_encryption_key;
    use tokio::time::sleep;

    use crate::auction::{
        create_bid, create_new_auction, erc20Contract, nftContract, reveal_winner,
    };
    use crate::config::Config;
    use crate::types::EthSigner;

    #[tokio::test]
    async fn test_mint() {
        let config_path = "../../config.toml".to_string();
        let config = Config::new(&config_path)
            .unwrap_or_else(|_| panic!("Failed to load config from {:?}", &config_path));
        let (owner_signer, _) = setup_wallet(&config, &env::var("OWNER_KEYSTORE").unwrap()).await;
        let (bidder_signer, _) = setup_wallet(&config, &env::var("BIDDER_KEYSTORE").unwrap()).await;

        let nft_contract_address =
            Address::from_str("0x4dab910a86affd90d888d82f16e9faaf33f5ee61").unwrap();

        let ntf_id =
            setup_asset(&config, &owner_signer, &bidder_signer, nft_contract_address).await;

        println!("New NFT ID: {}", ntf_id);
    }

    #[tokio::test]
    async fn test_auction_service() {
        let config_path = "config.toml".to_string();
        let config = Config::new(&config_path)
            .unwrap_or_else(|_| panic!("Failed to load config from {:?}", &config_path));

        let (owner_signer, owner_wallet) =
            setup_wallet(&config, &env::var("OWNER_KEYSTORE").unwrap()).await;
        let (bidder_signer, _bidder_wallet) =
            setup_wallet(&config, &env::var("BIDDER_KEYSTORE").unwrap()).await;

        let nft_contract_address =
            Address::from_str("0x4dab910a86affd90d888d82f16e9faaf33f5ee61").unwrap();

        let ntf_id =
            setup_asset(&config, &owner_signer, &bidder_signer, nft_contract_address).await;

        println!("New NFT ID: {}", ntf_id);

        let rpc_url = &config.chain.rpc_url;
        let network = Network::from_str(&config.chain.network).unwrap();
        let aligned_batcher_url = &config.chain.aligned_batcher_url;

        // Test create new auction success
        let name = "test".to_string();
        let description = "nothing".to_string();
        let encryption_key = get_encryption_key().unwrap();

        // Create new auction
        let auction_id = create_new_auction(
            owner_signer.clone(),
            config.contract_address,
            &encryption_key,
            name,
            description,
            nft_contract_address,
            ntf_id,
            U256::from(1000),
            U256::from(60),
        )
        .await
        .unwrap();

        // Create new bid
        create_bid(
            bidder_signer.clone(),
            config.contract_address,
            config.token_address,
            ntf_id,
            900,
        )
        .await
        .unwrap();

        println!("Sleep 60 second...");
        sleep(Duration::from_secs(60)).await;
        println!("Sleep over");

        // Get list bid
        reveal_winner(
            owner_signer.clone(),
            config.contract_address,
            auction_id,
            owner_wallet,
            rpc_url,
            network,
            aligned_batcher_url,
        )
        .await
        .unwrap();
    }

    async fn setup_asset(
        config: &Config,
        owner_signer: &EthSigner,
        bidder_signer: &EthSigner,
        nft_contract_address: Address,
    ) -> U256 {
        let ntf_id = U256::from(OsRng.next_u64());

        mint_ntf(
            owner_signer,
            owner_signer.address(),
            nft_contract_address,
            ntf_id,
        )
        .await;
        faucet_token(
            config,
            bidder_signer,
            bidder_signer.address(),
            U256::from(10000000u128),
        )
        .await;

        ntf_id
    }

    async fn faucet_token(
        config: &Config,
        signer: &SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
        wallet_address: Address,
        token_mint: U256,
    ) {
        // Set up token
        let token_contract = erc20Contract::new(config.token_address, signer.into());
        let token_contract_caller = token_contract.mint(wallet_address, token_mint);
        let token_tx = token_contract_caller.send().await.unwrap();
        let token_receipt = token_tx.await.unwrap().unwrap();
        println!("==========================================================================");
        println!("Faucet {} token successfully with:", token_mint);
        println!("Tx: {:?}", token_receipt.transaction_hash);
    }

    async fn mint_ntf(
        signer: &SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
        wallet_address: Address,
        nft_contract_address: Address,
        ntf_id: U256,
    ) {
        // Set up nft
        let erc721_contract = nftContract::new(nft_contract_address, signer.into());
        let nft_contract_caller = erc721_contract.mint(wallet_address, ntf_id);
        let nft_tx = nft_contract_caller.send().await.unwrap();
        let nft_receipt = nft_tx.await.unwrap().unwrap();
        println!("==========================================================================");
        println!("Mint NFT successfully with:");
        println!("Token ID: {:?}", ntf_id);
        println!("Tx: {:?}", nft_receipt.transaction_hash);
    }

    async fn setup_wallet(
        config: &Config,
        keystore_path: &String,
    ) -> (
        SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
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

        let path = if keystore_path.starts_with("~/") {
            let home_dir = home_dir().expect("Failed to get home directory");
            home_dir.join(keystore_path.strip_prefix("~/").unwrap())
        } else {
            PathBuf::from(keystore_path)
        };

        let wallet = LocalWallet::decrypt_keystore(path, &keystore_password)
            .expect("Failed to decrypt keystore")
            .with_chain_id(chain_id.as_u64());

        let signer = SignerMiddleware::new(Arc::new(provider), wallet.clone());

        (signer, wallet)
    }
}
