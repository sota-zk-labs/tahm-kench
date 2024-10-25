// use ethers::prelude::*;
// use ethers::providers::{Http, Provider};
// use ethers::signers::{LocalWallet, Signer};
// use ethers::types::Bytes;
// use home::home_dir;
//
// use crate::config::Config;
// use crate::entities::auction::{AssetEntity, AuctionEntity, BidEntity, WinnerEntity};
//
// #[tokio::test]
// async fn test_auction_service() {
//     let config_path = "config.toml".to_string();
//     let keystore_path = ".zk_auction/keystores/wallet_zk_auction".to_string();
//
//     let config =
//         Config::new(&config_path).expect(&format!("Failed to load config from {:?}", &config_path));
//
//     let rpc_url = &config.chain.rpc_url;
//     let provider =
//         Provider::<Http>::try_from(rpc_url.as_str()).expect("Failed to connect to provider");
//     let chain_id = provider
//         .get_chainid()
//         .await
//         .expect("Failed to get chain_id");
//
//     let keystore_password = rpassword::prompt_password("Enter keystore password: ")
//         .expect("Failed to read keystore password");
//
//     let home_dir = home_dir().expect("Failed to get home directory");
//     let path = home_dir.join(&keystore_path);
//     let wallet = LocalWallet::decrypt_keystore(path, &keystore_password)
//         .expect("Failed to decrypt keystore")
//         .with_chain_id(chain_id.as_u64());
//     let private_key = wallet.signer();
//     let public_key = private_key.verifying_key();
//     // Convert the public key to Bytes
//     let public_key_bytes = Bytes::from(public_key.to_encoded_point(false).as_ref().to_vec());
//
//     let signer = SignerMiddleware::new(provider.clone(), wallet.clone());
//
//     // Mock data
//     let mock_auction = AuctionEntity {
//         owner: wallet.address(),
//         owner_public_key: public_key_bytes,
//         asset: AssetEntity {
//             name: "Auction Draply".to_string(),
//             description: "German Dog".to_string(),
//             nft_contract_address: Default::default(),
//             token_id: U256(1),
//         },
//         winner: WinnerEntity {
//             winner_address: winner_.winner,
//             price: winner_.price,
//         },
//         deposit_price: 1,
//         end_time: 3600,
//         ended: false,
//     };
// }
