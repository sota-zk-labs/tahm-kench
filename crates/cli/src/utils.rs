use std::path::PathBuf;
use std::sync::Arc;

use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::{Middleware, SignerMiddleware};
use ethers::prelude::{Http, LocalWallet, Provider, Signer, Wallet};
use home::home_dir;

use crate::config::Config;

pub async fn setup_wallet(
    config: &Config,
    keystore_path: &String,
) -> (
    SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    Wallet<SigningKey>,
    SigningKey,
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

    let wallet = LocalWallet::decrypt_keystore(&path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    let signer = SignerMiddleware::new(Arc::new(provider), wallet.clone());

    let private_key = SigningKey::from_bytes(
        eth_keystore::decrypt_key(&path, &keystore_password)
            .unwrap()
            .as_slice()
            .into(),
    )
    .unwrap();
    (signer, wallet, private_key)
}
