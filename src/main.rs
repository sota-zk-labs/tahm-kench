use anyhow::{Context, Result};
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Bytes;

use crate::auction::{create_new_auction, get_auction, get_total_auction};

mod auction;
mod config;

/// TAHKEN
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
    #[clap(
        short,
        long,
        default_value = "/home/ubuntu/.foundry/keystores/wallet_zk_auction"
    )]
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
        target_price: U256,
        #[arg(short, long, default_value = "1")]
        time: u64,
    },
    /// Get list auctions opening
    ListAuctions,
    /// Get detail auctions
    GetAuctions {
        #[arg(short, long)]
        id_auction: U256,
    },
    /// Bid item
    Bid {
        #[arg(short, long)]
        price: i32,
        #[arg(short, long)]
        id_auction: i32,
    },
    /// Submit
    Submit {
        #[arg(short, long)]
        id: i32,
        #[arg(short, long)]
        private_key: Bytes,
    },
    /// Claim item
    Claim {
        #[arg(short, long)]
        id: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config = config::Config::new(&args.config_path).expect("Config env not set");

    let rpc_url = &config.chain.rpc_url;
    let provider =
        Provider::<Http>::try_from(rpc_url.as_str()).expect("Failed to connect to provider");
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to get chain_id");

    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");
    let wallet = LocalWallet::decrypt_keystore(&args.keystore_path, &keystore_password)
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
                println!("version 1.0");
                Ok(())
            }
            Commands::CreateAuction {
                name,
                description,
                target_price,
                time,
            } => {
                let duration = U256::from(time * 3600);
                let _ = create_new_auction(
                    signer,
                    &config.contract_address,
                    public_key_bytes,
                    name,
                    description,
                    target_price,
                    duration,
                )
                .await
                .context("Failed to create auction");
                Ok(())
            }
            Commands::ListAuctions => {
                let _ = get_total_auction(signer, &config.contract_address)
                    .await
                    .context("Failed to get total auction");
                Ok(())
            }
            Commands::GetAuctions { id_auction } => {
                let _ = get_auction(signer, &config.contract_address, id_auction)
                    .await
                    .context(format!("Failed to get auction with id: {}", id_auction));
                Ok(())
            }
            Commands::Bid { price, id_auction } => {
                println!(
                    "bid item with: (price: {}, id_auction: {})",
                    price, id_auction
                );
                Ok(())
            }
            Commands::Submit { id, private_key } => {
                println!("submit with: (id: {}, private_key: {})", id, private_key);
                Ok(())
            }
            Commands::Claim { id } => {
                println!("claim item: {}", id);
                Ok(())
            }
        },
        None => {
            Cli::command().print_help().unwrap();
            Ok(())
        }
    }
}
