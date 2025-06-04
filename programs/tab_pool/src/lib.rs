use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("Hvddzj6Z3eFJheuabJGQNuchCoo4LjwDAQizRVTqW31D");

#[program]
mod tab_pool {
    use super::*;

    /**
     * 
     */
    pub fn create_payment_job(ctx: Context<CreatePaymentJob>, payers: Vec<Pubkey>,payees: Vec<Pubkey>,amount: u64,deadline: Option<i64>) -> Result<()> {
        require!(payers.len() > 0, ErrorCode::InvalidInput);
        require!(payees.len() > 0, ErrorCode::InvalidInput);
        require!(amount > 0, ErrorCode::InvalidInput);

        let job = &mut ctx.accounts.job;
        job.authority = ctx.accounts.authority.key();
        job.amount = amount;
        job.deadline = deadline.unwrap_or(i64::MAX);
        job.closed = false;
        job.payees = payees;
        job.payers = payers.into_iter().map(|pk| WalletStatus { wallet: pk, paid: false }).collect();

        msg!("Payment job created with {} payers to {} as payee", job.payers.len(), job.payees.len());
        Ok(())
    }

    /**
    * 
    */
    pub fn pay(ctx: Context<Pay>) -> Result<()> {
        let payer_key = ctx.accounts.payer.key();
        let job = &mut ctx.accounts.job;
        require!(!job.closed, ErrorCode::AlreadyClosed);

        // find payer in job.payers
        let mut found = false;
        for ws in job.payers.iter_mut() {
            if ws.wallet == payer_key {
                require!(!ws.paid, ErrorCode::AlreadyPaid);
                // transfer SOL from payer to payee
                invoke(
                    &system_instruction::transfer(
                        &payer_key,
                        &job.key(),
                        job.amount,
                    ),
                    &[
                        ctx.accounts.payer.to_account_info(),
                        job.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;

                ws.paid = true;
                found = true;
                msg!("{} paid {} lamports", payer_key, job.amount);
                break;
            }
        }
        require!(found, ErrorCode::NotContributor);

        // Check if all contributors have paid - if yes, auto-distribute
        if job.contributors.iter().all(|c| c.paid) {
            msg!("All contributors have paid. Auto-distributing funds.");
            //return _distribute_funds(job, &ctx.accounts.system_program);
        }

        Ok(())
    }

    pub fn distribute_funds(ctx: Context<DistributeFunds>) -> Result<()> {}

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
    #[account(
        init,
        payer = authority,
        space = PaymentJob::space(payers.len(), payee.len()),
    )]
    pub job: Account<'info, PaymentJob>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Pay<'info> {
    #[account(mut)]
    pub job: Account<'info, PaymentJob>,
    
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeFunds<'info> {

}

// ==================== Account Structs ====================

#[account]
pub struct PaymentJob {
    pub authority: Pubkey, // Creator/Payee who controls the job
    pub amount: u64, // Amount payer owes
    pub deadline: i64, // Deadline for payment
    pub closed: bool, // Job closed
    pub payers: Vec<WalletStatus>, // Payers
    pub payees: Vec<Pubkey>, // Payee
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WalletStatus {
    pub wallet: Pubkey, // The payer's wallet
    pub paid: bool, // Whether the payer has paid
}

impl PaymentJob {
    pub fn space(num_payers: usize, num_payees: usize) -> usize {
        8 +
        32 +
        8 +
        8 +
        1 +
        4 + num_payers * std::mem::size_of::<WalletStatus>() +
        4 + num_payees * 32  
    }
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