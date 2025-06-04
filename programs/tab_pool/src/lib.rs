use anchor_lang::prelude::*;

declare_id!("Hvddzj6Z3eFJheuabJGQNuchCoo4LjwDAQizRVTqW31D");

#[program]
mod tab_pool {
    use super::*;

    /**
     * 
     */
    pub fn create_payment_job(ctx: Context<CreatePaymentJob>, payers: Vec<Pubkey>,payee: Vec<Pubkey>,amount: u64,deadline: Option<u64>) -> Result<()> {
        require!(payers.len() > 0, ErrorCode::InvalidInput);
        require!(recipients.len() > 0, ErrorCode::InvalidInput);
        require!(amount > 0, ErrorCode::InvalidInput);
        Ok(())
    }

    /**
    * 
    */
    pub fn pay(ctx: Context<Pay>) -> Result<()> {
        Ok(())
    }

}

// ==================== Account Structs ====================

#[derive(Accounts)]
#[instruction(
    payers: Vec<Pubkey>,
    payee: Vec<Pubkey>,
    amount: u64,
    deadline: Option<u64>,
)]
pub struct CreatePaymentJob<'info> {
    pub job: Account<'info, PaymentJob>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Pay<'info> {
    pub job: Account<'info, PaymentJob>,
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ==================== Account Structs ====================

#[account]
pub struct PaymentJob {
    pub authority: Pubkey, // Creator/Payee who controls the job
    pub amount: u64, // Amount payer owes
    pub deadline: i64, // Deadline for payment
    pub closed: bool, // Job closed
    pub payers: Vec<WalletStatus>, // Payers
    pub payee: Vec<Pubkey>, // Payee
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WalletStatus {
    pub wallet: Pubkey, // The payer's wallet
    pub paid: bool, // Whether the payer has paid
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid input parameters")]
    InvalidInput,
    #[msg("Contributor has already paid")]
    AlreadyPaid,
    #[msg("Contributor is not listed")]
    NotContributor,
}