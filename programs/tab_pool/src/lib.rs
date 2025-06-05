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
        // Check if job is closed first
        require!(!ctx.accounts.job.closed, ErrorCode::AlreadyClosed);

        // Store payer key
        let payer_key = ctx.accounts.payer.key();
        
        // Find payer index in job.payers without mutable borrow
        let mut payer_index = None;
        for (i, ws) in ctx.accounts.job.payers.iter().enumerate() {
            if ws.wallet == payer_key {
                require!(!ws.paid, ErrorCode::AlreadyPaid);
                payer_index = Some(i);
                break;
            }
        }

        // Check if payer was found
        let payer_index = payer_index.ok_or(error!(ErrorCode::NotContributor))?;

        // Get key and amount before transfer
        let job_key = ctx.accounts.job.key();
        let amount = ctx.accounts.job.amount;

        // Transfer SOL from payer to job account
        invoke(
            &system_instruction::transfer(
                &payer_key,
                &job_key,
                amount,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.job.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Now get mutable reference and mark payer as paid
        ctx.accounts.job.payers[payer_index].paid = true;
        msg!("{} paid {} lamports", payer_key, amount);

        // Check if all contributors have paid - if yes, auto-distribute
        if ctx.accounts.job.payers.iter().all(|c| c.paid) {
            msg!("All contributors have paid. Auto-distributing funds.");
            //return _distribute_funds(job, &ctx.accounts.system_program);
        }

        Ok(())
    }

    pub fn distribute_funds(ctx: Context<DistributeFunds>) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(!job.closed, ErrorCode::AlreadyClosed);

        // If authority is not the signer, check deadline
        if !ctx.accounts.authority.key().eq(&job.authority) || !ctx.accounts.authority.is_signer {
            let clock = Clock::get()?;
            require!(clock.unix_timestamp >= job.deadline, ErrorCode::BeforeDeadline);
        }

        // Count paid contributors
        let paid_count = job.payers.iter().filter(|p| p.paid).count();
        if paid_count == 0 {
            job.closed = true;
            return Ok(());
        }

        // Calculate amounts
        let total_collected = paid_count as u64 * job.amount;
        let current_job_lamports = job.to_account_info().lamports();
        require!(current_job_lamports >= total_collected, ErrorCode::InsufficientFunds);

        let distributable_amount = std::cmp::min(total_collected, current_job_lamports);
        let per_payee = distributable_amount / job.payees.len() as u64;

        if per_payee > 0 {
            // Clone payees to avoid borrow checker issues
            let payees = job.payees.clone();
            
            // Mark job as closed before transfers
            job.closed = true;

            // Process transfers
            for payee in payees.iter() {
                // Create the instruction
                let ix = system_instruction::transfer(
                    &job.key(),
                    payee, 
                    per_payee
                );
                
                // Process the CPI
                let accounts = &[
                    job.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ];
                
                anchor_lang::solana_program::program::invoke(&ix, accounts)?;
                msg!("Transferred {} lamports to payee {}", per_payee, payee);
            }
            
            msg!("Payment job closed and funds distributed");
        } else {
            job.closed = true;
            msg!("Payment job closed (no funds to distribute)");
        }
        
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
    #[account(mut)]
    pub job: Account<'info, PaymentJob>,

    pub authority: UncheckedAccount<'info>,

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
    #[msg("Payment job is already closed")]
    AlreadyClosed,
    #[msg("Cannot distribute before deadline unless authority")]
    BeforeDeadline,
    #[msg("Insufficient funds in job account")]
    InsufficientFunds,
}