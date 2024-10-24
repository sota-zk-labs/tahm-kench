use anyhow::{Context, Result};
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Bytes;
use home::home_dir;

use crate::auction_service::{create_bid, create_new_auction, get_auction, get_total_auction};

mod auction_service;
mod config;
mod entity;

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
    /// Path of local wallet
    #[clap(short, long, default_value = ".zk_auction/keystores/wallet_zk_auction")]
    keystore_path: String,
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
        token_id: U256,
        #[arg(short, long)]
        target_price: U256,
        #[arg(short, long, default_value = "1")]
        time: u64,
    },
    /// Get detail auctions
    GetAuction {
        #[arg(short, long)]
        auction_id: U256,
    },
    /// Get list auctions opening
    ListAuctions,
    /// Bid item
    Bid {
        #[arg(short, long)]
        price: U256,
        #[arg(short, long)]
        auction_id: U256,
    },
    /// Reveal winner
    RevealWinner {
        #[arg(short, long)]
        id: i32,
        #[arg(short, long)]
        private_key: Bytes,
    },
    /// Withdraw deposit token
    Withdraw {
        #[arg(short, long)]
        auction_id: U256,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.version {
        println!(
            "{:?}",
            std::env::var("CARGO_PKG_VERSION").unwrap_or_default()
        );
        return Ok(());
    }

    let config = config::Config::new(&args.config_path).expect(&format!(
        "Failed to load config from {:?}",
        &args.config_path
    ));

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
    let path = home_dir.join(&args.keystore_path);
    let wallet = LocalWallet::decrypt_keystore(path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    let private_key = wallet.signer();
    let public_key = private_key.verifying_key();
    // Convert the public key to Bytes
    let public_key_bytes = Bytes::from(public_key.to_encoded_point(false).as_ref().to_vec());

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());

    match args.command {
        Some(command) => match command {
            Commands::Version => {
                println!(
                    "{:?}",
                    std::env::var("CARGO_PKG_VERSION").unwrap_or_default()
                );
                Ok(())
            }
            Commands::CreateAuction {
                name,
                description,
                nft_contract_address,
                token_id,
                target_price,
                time,
            } => {
                let duration = U256::from(time * 3600);
                let _ = create_new_auction(
                    signer,
                    config.contract_address,
                    public_key_bytes,
                    name,
                    description,
                    nft_contract_address,
                    token_id,
                    target_price,
                    duration,
                )
                .await
                .context("Failed to create auction");
                Ok(())
            }
            Commands::GetAuction { auction_id } => {
                let _ = get_auction(signer, config.contract_address, auction_id)
                    .await
                    .context(format!("Failed to get auction with id: {}", auction_id));
                Ok(())
            }
            Commands::ListAuctions => {
                let _ = get_total_auction(signer, config.contract_address)
                    .await
                    .context("Failed to get total auction");
                Ok(())
            }
            Commands::Bid { price, auction_id } => {
                let _ = create_bid(signer, config.contract_address, config.token_address, auction_id, price)
                    .await
                    .context(format!("Failed to bid auction with id: {}", auction_id));
                Ok(())
            }
            Commands::RevealWinner { id, private_key } => {
                println!("submit with: (id: {}, private_key: {})", id, private_key);
                Ok(())
            }
            Commands::Withdraw { auction_id } => {
                println!("withdraw");
                Ok(())
            }
        },
        None => {
            Cli::command().print_help().unwrap();
            Ok(())
        }
    }
}
