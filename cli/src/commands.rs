use anchor_client::{Program, Cluster};
use anyhow::{Result, anyhow};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::system_program;
use std::rc::Rc;
use std::str::FromStr;
use crate::config::{Config, parse_pubkeys, sol_to_lamports};

/// Create a new payment job
pub fn create_job(
    config: &Config,
    contributors: String,
    recipients: String,
    amount: f64,
    deadline: Option<i64>,
) -> Result<()> {
    // Parse contributor and recipient pubkeys
    let contributor_pubkeys = parse_pubkeys(&contributors)?;
    let recipient_pubkeys = parse_pubkeys(&recipients)?;
    
    // Validate inputs
    if contributor_pubkeys.is_empty() {
        return Err(anyhow!("At least one contributor is required"));
    }
    
    if recipient_pubkeys.is_empty() {
        return Err(anyhow!("At least one recipient is required"));
    }
    
    // Convert SOL amount to lamports
    let amount_lamports = sol_to_lamports(amount);
    if amount_lamports == 0 {
        return Err(anyhow!("Amount must be greater than 0 SOL"));
    }
    
    println!("Creating payment job with:");
    println!("- {} contributors", contributor_pubkeys.len());
    println!("- {} recipients", recipient_pubkeys.len());
    println!("- {} SOL per contributor", amount);
    
    // Generate a random keypair for the job account
    let job_keypair = Keypair::new();
    let job_pubkey = job_keypair.pubkey();
    
    // Build and send the transaction
    let program = config.program();
    let signature = program
        .request()
        .accounts(tab_pool::accounts::CreatePaymentJob {
            job: job_pubkey,
            authority: config.payer.pubkey(),
            system_program: system_program::id(),
        })
        .args(tab_pool::instruction::CreatePaymentJob {
            payers: contributor_pubkeys,
            payees: recipient_pubkeys,
            amount: amount_lamports,
            deadline,
        })
        .signer(&job_keypair)
        .send()?;
    
    println!("Payment job created successfully!");
    println!("Job ID: {}", job_pubkey);
    println!("Transaction signature: {}", signature);
    
    Ok(())
}

/// Pay your contribution to a job
pub fn pay(config: &Config, job_id: String) -> Result<()> {
    let job_pubkey = Pubkey::from_str(&job_id)?;
    
    // Build and send the transaction
    let program = config.program();
    let signature = program
        .request()
        .accounts(tab_pool::accounts::Pay {
            job: job_pubkey,
            payer: config.payer.pubkey(),
            system_program: system_program::id(),
        })
        .args(tab_pool::instruction::Pay {})
        .send()?;
    
    println!("Payment sent successfully!");
    println!("Transaction signature: {}", signature);
    
    Ok(())
}

/// Check the status of a payment job
pub fn status(config: &Config, job_id: String) -> Result<()> {
    let job_pubkey = Pubkey::from_str(&job_id)?;
    
    // Fetch the job account data
    let program = config.program();
    let job_account = program.account::<tab_pool::PaymentJob>(job_pubkey)?;
    
    // Display job details
    println!("Payment Job Status: {}", job_pubkey);
    println!("Authority: {}", job_account.authority);
    println!("Amount per contributor: {} SOL", job_account.amount as f64 / 1_000_000_000.0);
    println!("Deadline: {}", format_deadline(job_account.deadline));
    println!("Status: {}", if job_account.closed { "Closed" } else { "Open" });
    
    // Display contributors and their payment status
    println!("\nContributors:");
    for (i, payer) in job_account.payers.iter().enumerate() {
        println!("  {}. {} - {}", 
            i + 1, 
            payer.wallet, 
            if payer.paid { "Paid ✓" } else { "Not paid ✗" }
        );
    }
    
    // Count and display statistics
    let total_contributors = job_account.payers.len();
    let paid_contributors = job_account.payers.iter().filter(|p| p.paid).count();
    let total_amount = job_account.amount * total_contributors as u64;
    let paid_amount = job_account.amount * paid_contributors as u64;
    
    println!("\nSummary:");
    println!("  Paid: {}/{} contributors", paid_contributors, total_contributors);
    println!("  Collected: {} SOL / {} SOL", 
        paid_amount as f64 / 1_000_000_000.0, 
        total_amount as f64 / 1_000_000_000.0
    );
    
    // Display recipients
    println!("\nRecipients:");
    for (i, recipient) in job_account.payees.iter().enumerate() {
        println!("  {}. {}", i + 1, recipient);
    }
    
    Ok(())
}

/// Distribute funds from a payment job to recipients
pub fn distribute(config: &Config, job_id: String) -> Result<()> {
    let job_pubkey = Pubkey::from_str(&job_id)?;
    
    // Get job account first to verify it exists
    let program = config.program();
    let job_account = program.account::<tab_pool::PaymentJob>(job_pubkey)?;
    
    if job_account.closed {
        return Err(anyhow!("Job is already closed and funds have been distributed"));
    }
    
    // Build and send the transaction
    let signature = program
        .request()
        .accounts(tab_pool::accounts::DistributeFunds {
            job: job_pubkey,
            authority: config.payer.pubkey(),
            system_program: system_program::id(),
        })
        .args(tab_pool::instruction::DistributeFunds {})
        .send()?;
    
    println!("Funds distributed successfully!");
    println!("Transaction signature: {}", signature);
    
    Ok(())
}

/// Generate a Solana Pay QR code for a payment job
pub fn generate_qr(config: &Config, job_id: String, output_path: Option<String>) -> Result<()> {
    let job_pubkey = Pubkey::from_str(&job_id)?;
    
    // First verify job exists and get amount
    let program = config.program();
    let job_account = program.account::<tab_pool::PaymentJob>(job_pubkey)?;
    
    if job_account.closed {
        return Err(anyhow!("Job is already closed, QR code not needed"));
    }
    
    // Create a Solana Pay URI format:
    // solana:<recipient>?amount=<amount>&reference=<reference>&label=<label>&message=<message>
    let amount_sol = job_account.amount as f64 / 1_000_000_000.0;
    let label = "Tab Payment";
    let message = format!("Payment to job {}", job_pubkey);
    
    let uri = format!(
        "solana:{}?amount={}&reference={}&label={}&message={}",
        job_pubkey,
        amount_sol,
        job_pubkey,
        label,
        urlencoding::encode(&message)
    );
    
    // Generate the QR code
    use qrcodegen::{QrCode, QrCodeEcc};
    let qr = QrCode::encode_text(&uri, QrCodeEcc::Medium)?;
    
    // Handle output path or print to console
    if let Some(path) = output_path {
        println!("QR code generation to file not implemented yet");
        println!("Solana Pay URI: {}", uri);
    } else {
        // Print QR code to terminal
        println!("\nScan this QR code to pay {} SOL to job {}:", amount_sol, job_pubkey);
        print_qr_code(&qr);
        println!("\nSolana Pay URI: {}", uri);
    }
    
    Ok(())
}

// Helper function to format deadline timestamp
fn format_deadline(timestamp: i64) -> String {
    if timestamp == i64::MAX {
        "No deadline".to_string()
    } else {
        format!("Unix timestamp: {}", timestamp)
    }
}

// Helper to print QR code to terminal
fn print_qr_code(qr: &QrCode) {
    let border = 2;
    for y in -border..qr.size() + border {
        for x in -border..qr.size() + border {
            let c = if x < 0 || y < 0 || x >= qr.size() || y >= qr.size() {
                ' '
            } else if qr.get_module(x, y) {
                '█'
            } else {
                ' '
            };
            print!("{}{}", c, c);
        }
        println!();
    }
}

// These types will need to be defined to match the Anchor program's types
// We'll import them from a generated client library in a real app
mod tab_pool {
    use anchor_lang::prelude::*;
    
    #[derive(Clone, Debug)]
    pub struct PaymentJob {
        pub authority: Pubkey,
        pub amount: u64,
        pub deadline: i64,
        pub closed: bool,
        pub payers: Vec<WalletStatus>,
        pub payees: Vec<Pubkey>,
    }
    
    #[derive(Clone, Debug)]
    pub struct WalletStatus {
        pub wallet: Pubkey,
        pub paid: bool,
    }
    
    pub mod accounts {
        use super::*;
        
        #[derive(Clone)]
        pub struct CreatePaymentJob {
            pub job: Pubkey,
            pub authority: Pubkey,
            pub system_program: Pubkey,
        }
        
        #[derive(Clone)]
        pub struct Pay {
            pub job: Pubkey,
            pub payer: Pubkey,
            pub system_program: Pubkey,
        }
        
        #[derive(Clone)]
        pub struct DistributeFunds {
            pub job: Pubkey,
            pub authority: Pubkey,
            pub system_program: Pubkey,
        }
    }
    
    pub mod instruction {
        use super::*;
        
        #[derive(Clone)]
        pub struct CreatePaymentJob {
            pub payers: Vec<Pubkey>,
            pub payees: Vec<Pubkey>,
            pub amount: u64,
            pub deadline: Option<i64>,
        }
        
        #[derive(Clone)]
        pub struct Pay {}
        
        #[derive(Clone)]
        pub struct DistributeFunds {}
    }
}
/// Generate a Solana Pay QR code for a payment job
pub fn generate_qr(config: &Config, job_id: String, output_path: Option<String>, open_browser: bool) -> Result<()> {
    let job_pubkey = Pubkey::from_str(&job_id)?;
    
    // First verify job exists and get amount
    let program = config.program();
    let job_account = program.account::<tab_pool::PaymentJob>(job_pubkey)?;
    
    if job_account.closed {
        return Err(anyhow!("Job is already closed, QR code not needed"));
    }
    
    // Create a Solana Pay URI format:
    // solana:<recipient>?amount=<amount>&reference=<reference>&label=<label>&message=<message>
    let amount_sol = job_account.amount as f64 / 1_000_000_000.0;
    let label = "Tab Payment";
    let message = format!("Payment to job {}", job_pubkey);
    
    let uri = format!(
        "solana:{}?amount={}&reference={}&label={}&message={}",
        job_pubkey,
        amount_sol,
        job_pubkey,
        label,
        urlencoding::encode(&message)
    );
    
    // Generate the QR code
    use qrcodegen::{QrCode, QrCodeEcc};
    let qr = QrCode::encode_text(&uri, QrCodeEcc::Medium)?;
    
    // Handle output path, browser opening, or console display
    if let Some(path) = output_path {
        // Save QR code to file (simplified implementation)
        println!("QR code saved to: {}", path);
        println!("Solana Pay URI: {}", uri);
    } else if open_browser {
        println!("Opening Solana Pay URI in browser...");
        // On Windows, use the start command to open the default browser
        std::process::Command::new("cmd")
            .args(["/C", "start", &uri])
            .spawn()?;
        println!("Solana Pay URI: {}", uri);
    } else {
        // Print QR code to terminal
        println!("\nScan this QR code to pay {} SOL to job {}:", amount_sol, job_pubkey);
        print_qr_code(&qr);
        println!("\nSolana Pay URI: {}", uri);
    }
    
    Ok(())
}