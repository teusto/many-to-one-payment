use anchor_client::{Client, Cluster};
use anyhow::{Result, anyhow};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, read_keypair_file};
use std::rc::Rc;
use std::path::Path;
use std::str::FromStr;

// Program ID from Anchor.toml
pub const PROGRAM_ID: &str = "Hvddzj6Z3eFJheuabJGQNuchCoo4LjwDAQizRVTqW31D";

pub struct Config {
    pub client: Client,
    pub payer: Rc<Keypair>,
    pub program_id: Pubkey,
}

impl Config {
    pub fn new(url: Option<String>, keypair_path: Option<String>) -> Result<Self> {
        // Get cluster RPC URL (default to localhost)
        let url = url.unwrap_or_else(|| "http://localhost:8899".to_string());
        let cluster = Cluster::Custom(url, "ws://localhost:8900".to_string());

        // Get wallet keypair (default to ~/.config/solana/id.json)
        let keypair_path = keypair_path.unwrap_or_else(|| {
            shellexpand::tilde("~/.config/solana/id.json").to_string()
        });

        // Load keypair from file
        let payer = match read_keypair_file(&keypair_path) {
            Ok(keypair) => Rc::new(keypair),
            Err(_) => {
                return Err(anyhow!("Failed to read keypair from {}", keypair_path));
            }
        };

        // Create anchor client
        let client = Client::new_with_options(
            cluster,
            payer.clone(),
            CommitmentConfig::confirmed(),
        );

        // Parse program ID
        let program_id = Pubkey::from_str(PROGRAM_ID)?;

        Ok(Self {
            client,
            payer,
            program_id,
        })
    }

    // Helper method to get program client
    pub fn program(&self) -> anchor_client::Program<Rc<Keypair>> {
        self.client.program(self.program_id)
    }
}

// Parse a pubkey from string with helpful error
pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey> {
    Pubkey::from_str(pubkey_str)
        .map_err(|_| anyhow!("Invalid Solana address: {}", pubkey_str))
}

// Parse a comma-separated list of pubkeys
pub fn parse_pubkeys(pubkeys_str: &str) -> Result<Vec<Pubkey>> {
    pubkeys_str
        .split(',')
        .map(|s| parse_pubkey(s.trim()))
        .collect()
}

// Convert SOL to lamports
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}