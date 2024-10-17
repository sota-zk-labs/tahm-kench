use serde_derive::Deserialize;
pub const REGISTRY_URL: &str = "https://crates.io";

#[derive(Deserialize, Debug, Clone)]
struct Version {
    num: String,
}