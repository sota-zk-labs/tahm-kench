use config::{Config as ConfigLoader, File, FileFormat};
use ethers::types::H160;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub rpc_url: String,
    // pub etherscan_api_key: String
    // pub network:
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub chain: ChainConfig,
    pub contract_address: H160,
}

impl Config {
    pub fn new(config_path: &String) -> Result<Config, config::ConfigError> {
        let content = ConfigLoader::builder()
            .add_source(File::new(config_path, FileFormat::Toml))
            .build()?;
        let config: Config = content.try_deserialize::<Config>()?;

        Ok(config)
    }
}
