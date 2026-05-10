use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("9e8CP55VmG48GRvLQCq9ytrofocjWy4YtCsgSSonM96R");

#[program]
pub mod escrow {
    use super::*;

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        job_id: u64,
        freelancer: Pubkey,
        amount: u64,
        deadline: i64,
        description: String,
    ) -> Result<()> {
        let clock = Clock::get()?;
        require!(amount >= 1_000_000, EscrowError::AmountTooSmall);
        require!(deadline > clock.unix_timestamp, EscrowError::DeadlineInPast);
        require!(deadline <= clock.unix_timestamp + 365 * 24 * 3600, EscrowError::DeadlineTooFar);
        require!(!description.trim().is_empty(), EscrowError::EmptyDescription);
        require!(description.as_bytes().len() <= EscrowJob::MAX_DESCRIPTION_BYTES, EscrowError::DescriptionTooLong);
        require!(freelancer != ctx.accounts.client.key(), EscrowError::SelfAssignment);

        let escrow_job = &mut ctx.accounts.escrow_job;
        escrow_job.job_id = job_id;
        escrow_job.client = ctx.accounts.client.key();
        escrow_job.freelancer = freelancer;
        escrow_job.amount = amount;
        escrow_job.deadline = deadline;
        escrow_job.status = EscrowStatus::Open;
        escrow_job.description = description;
        escrow_job.bump = ctx.bumps.escrow_job;

        let transfer_accounts = system_program::Transfer {
            from: ctx.accounts.client.to_account_info(),
            to: ctx.accounts.escrow_job.to_account_info(),
        };
        let transfer_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_accounts,
        );
        system_program::transfer(transfer_context, amount)?;
        Ok(())
    }

    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        let escrow_job = &mut ctx.accounts.escrow_job;
        require!(escrow_job.status == EscrowStatus::Open, EscrowError::InvalidStatus);
        escrow_job.status = EscrowStatus::InProgress;
        Ok(())
    }

    pub fn submit_work(ctx: Context<SubmitWork>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow_job = &mut ctx.accounts.escrow_job;
        require!(escrow_job.status == EscrowStatus::InProgress, EscrowError::InvalidStatus);
        require!(clock.unix_timestamp <= escrow_job.deadline, EscrowError::DeadlinePassed);
        escrow_job.status = EscrowStatus::Submitted;
        Ok(())
    }

    pub fn approve_release(ctx: Context<ApproveRelease>) -> Result<()> {
        let escrow_job = &mut ctx.accounts.escrow_job;
        require!(escrow_job.status == EscrowStatus::Submitted, EscrowError::InvalidStatus);
        let amount = escrow_job.amount;
        **ctx.accounts.escrow_job.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.freelancer.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow_job = &mut ctx.accounts.escrow_job;
        let can_refund = escrow_job.status == EscrowStatus::Open || escrow_job.status == EscrowStatus::InProgress;
        require!(can_refund, EscrowError::InvalidStatus);
        require!(clock.unix_timestamp > escrow_job.deadline, EscrowError::DeadlineNotPassed);
        let amount = escrow_job.amount;
        **ctx.accounts.escrow_job.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.client.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>) -> Result<()> {
        let escrow_job = &mut ctx.accounts.escrow_job;
        let caller = ctx.accounts.caller.key();
        require!(caller == escrow_job.client || caller == escrow_job.freelancer, EscrowError::UnauthorizedDisputer);
        require!(escrow_job.status == EscrowStatus::Submitted, EscrowError::InvalidStatus);
        escrow_job.status = EscrowStatus::Disputed;
        Ok(())
    }

    pub fn resolve_dispute(ctx: Context<ResolveDispute>, pay_freelancer: bool) -> Result<()> {
        let escrow_job = &mut ctx.accounts.escrow_job;
        require!(escrow_job.status == EscrowStatus::Disputed, EscrowError::InvalidStatus);
        let amount = escrow_job.amount;
        if pay_freelancer {
            **ctx.accounts.escrow_job.to_account_info().try_borrow_mut_lamports()? -= amount;
            **ctx.accounts.freelancer.to_account_info().try_borrow_mut_lamports()? += amount;
        } else {
            **ctx.accounts.escrow_job.to_account_info().try_borrow_mut_lamports()? -= amount;
            **ctx.accounts.client.to_account_info().try_borrow_mut_lamports()? += amount;
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = client,
        space = EscrowJob::SPACE,
        seeds = [b"escrow", client.key().as_ref(), job_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    #[account(mut)]
    pub client: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump,
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    pub freelancer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump,
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    pub freelancer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ApproveRelease<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump,
        has_one = client @ EscrowError::UnauthorizedClient,
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer,
        close = client
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRefund<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump,
        has_one = client @ EscrowError::UnauthorizedClient,
        close = client
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    #[account(mut)]
    pub client: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RaiseDispute<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()],
        bump = escrow_job.bump,
        close = client
    )]
    pub escrow_job: Account<'info, EscrowJob>,
    #[account(mut)]
    pub client: SystemAccount<'info>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    pub arbitrator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct EscrowJob {
    pub job_id: u64,
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
    pub deadline: i64,
    pub status: EscrowStatus,
    pub description: String,
    pub bump: u8,
}

impl EscrowJob {
    pub const MAX_DESCRIPTION_BYTES: usize = 200;
    pub const DISCRIMINATOR_BYTES: usize = 8;
    pub const U64_BYTES: usize = 8;
    pub const PUBKEY_BYTES: usize = 32;
    pub const I64_BYTES: usize = 8;
    pub const ENUM_BYTES: usize = 1;
    pub const STRING_PREFIX_BYTES: usize = 4;
    pub const U8_BYTES: usize = 1;
    pub const SPACE: usize = Self::DISCRIMINATOR_BYTES + Self::U64_BYTES + Self::PUBKEY_BYTES + Self::PUBKEY_BYTES + Self::U64_BYTES + Self::I64_BYTES + Self::ENUM_BYTES + Self::STRING_PREFIX_BYTES + Self::MAX_DESCRIPTION_BYTES + Self::U8_BYTES;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    Open,
    InProgress,
    Submitted,
    Completed,
    Disputed,
    Refunded,
}

#[error_code]
pub enum EscrowError {
    #[msg("The escrow amount must be at least 0.001 SOL.")]
    AmountTooSmall,
    #[msg("The deadline must be in the future.")]
    DeadlineInPast,
    #[msg("The deadline cannot be more than 1 year in the future.")]
    DeadlineTooFar,
    #[msg("The deadline has passed, work can no longer be submitted.")]
    DeadlinePassed,
    #[msg("The deadline has not passed yet.")]
    DeadlineNotPassed,
    #[msg("The job description cannot be empty.")]
    EmptyDescription,
    #[msg("The job description cannot be longer than 200 bytes.")]
    DescriptionTooLong,
    #[msg("The client and freelancer cannot be the same wallet.")]
    SelfAssignment,
    #[msg("Only the client for this escrow job can perform this action.")]
    UnauthorizedClient,
    #[msg("Only the assigned freelancer for this escrow job can perform this action.")]
    UnauthorizedFreelancer,
    #[msg("Only the client or assigned freelancer can raise a dispute.")]
    UnauthorizedDisputer,
    #[msg("This action is not allowed in the escrow job's current status.")]
    InvalidStatus,
    #[msg("A lamport balance calculation failed.")]
    MathOverflow,
}
