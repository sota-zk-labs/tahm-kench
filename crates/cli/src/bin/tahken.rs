use std::str::FromStr;
use std::sync::Arc;

use aligned_sdk::core::types::Network;
use anyhow::Result;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::Provider;
use ethers::signers::{LocalWallet, Signer};
use prover_sdk::get_encryption_key;
use zk_auction::auction::{
    create_bid, create_new_auction, get_auction, get_total_auction, reveal_winner, withdraw,
};
use zk_auction::config::Config;

#[derive(Parser, Debug)]
#[command(name = "tahken")]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(short, long)]
    version: bool,
    #[clap(short, long, default_value = "config.toml")]
    config_path: String,
}

#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Commands {
    /// Print current version
    Version,
    /// Create auction session
    CreateAuction {
        #[arg(short, long)]
        name: String,
        #[arg(short, long, default_value = "")]
        description: String,
        #[arg(short, long)]
        nft_contract_address: Address,
        #[arg(short, long)]
        token_id: u128,
        #[arg(short, long)]
        target_price: u128,
        #[arg(short, long, default_value = "3600")]
        time: u128,
        #[clap(short, long)]
        keystore_path: String,
        #[arg(
            short,
            long,
            default_value = "0xd6a367e96abd5872f0e39b9f5df0ed1cd125c41e"
        )]
        token_address: Address,
    },
    /// Get detail auctions
    GetAuction {
        #[arg(short, long)]
        auction_id: u128,
        #[clap(short, long)]
        keystore_path: String,
    },
    /// Get total auctions
    ListAuctions {
        #[clap(short, long)]
        keystore_path: String,
    },
    /// Bid item
    Bid {
        #[arg(short, long)]
        price: u128,
        #[arg(short, long)]
        auction_id: u128,
        #[clap(short, long)]
        keystore_path: String,
    },
    /// Reveal winner
    RevealWinner {
        #[arg(short, long)]
        auction_id: u128,
        #[clap(short, long)]
        keystore_path: String,
    },
    /// Withdraw deposit token
    Withdraw {
        #[arg(short, long)]
        auction_id: u128,
        #[clap(short, long)]
        keystore_path: String,
    },
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.version {
        println!(env!("APP_VERSION"));
        return Ok(());
    }

    let config = Config::new(&args.config_path)
        .unwrap_or_else(|_| panic!("Failed to load config from {:?}", &args.config_path));

    let rpc_url = config.chain.rpc_url.as_str();
    let network = Network::from_str(&config.chain.network).unwrap();
    let aligned_batcher_url = config.chain.aligned_batcher_url.as_str();

    match args.command {
        Some(command) => match command {
            Commands::Version => {
                println!(env!("APP_VERSION"));
                Ok(())
            }
            Commands::CreateAuction {
                name,
                description,
                nft_contract_address,
                token_id,
                target_price,
                time,
                keystore_path,
                token_address,
            } => {
                let (signer, _wallet_address, _wallet) =
                    set_up_wallet(config.clone(), keystore_path).await;
                let encryption_key = get_encryption_key()?;
                create_new_auction(
                    signer,
                    config.contract_address,
                    &encryption_key,
                    token_address,
                    name,
                    description,
                    nft_contract_address,
                    U256::from(token_id),
                    U256::from(target_price),
                    U256::from(time),
                )
                .await
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    panic!("Failed to create auction");
                });
                Ok(())
            }
            Commands::GetAuction {
                auction_id,
                keystore_path,
            } => {
                let (signer, _wallet_address, _wallet) =
                    set_up_wallet(config.clone(), keystore_path).await;
                get_auction(signer, config.contract_address, U256::from(auction_id))
                    .await
                    .unwrap_or_else(|e| {
                        println!("{}", e);
                        panic!("Failed to get auction with id: {}", auction_id);
                    });
                Ok(())
            }
            Commands::ListAuctions { keystore_path } => {
                let (signer, _, _) = set_up_wallet(config.clone(), keystore_path).await;
                get_total_auction(signer, config.contract_address)
                    .await
                    .unwrap_or_else(|e| {
                        println!("{}", e);
                        panic!("Failed to get total auction");
                    });
                Ok(())
            }
            Commands::Bid {
                price,
                auction_id,
                keystore_path,
            } => {
                let (signer, _, _) = set_up_wallet(config.clone(), keystore_path).await;
                create_bid(
                    signer,
                    config.contract_address,
                    U256::from(auction_id),
                    price,
                )
                .await
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    panic!("Failed to bid auction with id: {}", auction_id);
                });
                Ok(())
            }
            Commands::RevealWinner {
                auction_id,
                keystore_path,
            } => {
                let (signer, _, wallet) = set_up_wallet(config.clone(), keystore_path).await;
                reveal_winner(
                    signer,
                    config.contract_address,
                    U256::from(auction_id),
                    wallet,
                    rpc_url,
                    network,
                    aligned_batcher_url,
                )
                .await
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    panic!("Failed to reveal winner of auction with id: {}", auction_id);
                });
                Ok(())
            }
            Commands::Withdraw {
                auction_id,
                keystore_path,
            } => {
                let (signer, _, _) = set_up_wallet(config.clone(), keystore_path).await;
                withdraw(signer, config.contract_address, U256::from(auction_id))
                    .await
                    .unwrap_or_else(|e| {
                        println!("{}", e);
                        panic!("Failed to withdraw from auction with id: {}", auction_id);
                    });
                Ok(())
            }
        },
        None => {
            Cli::command().print_help()?;
            Ok(())
        }
    }
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

    let wallet = LocalWallet::decrypt_keystore(keystore_path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    let signer = SignerMiddleware::new(Arc::new(provider.clone()), wallet.clone());
    let wallet_address = signer.clone().address();

    (signer, wallet_address, wallet)
}
