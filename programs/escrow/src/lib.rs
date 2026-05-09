use anchor_lang::prelude::*; // Imports Anchor's most common Solana types, macros, and helper functions.
use anchor_lang::system_program; // Imports Anchor's wrapper around Solana's native system program transfers.

declare_id!("9e8CP55VmG48GRvLQCq9ytrofocjWy4YtCsgSSonM96R"); // Sets this program's public key placeholder; replace it with your real deployed program ID later.

#[program] // Tells Anchor that the module below contains the public instruction handlers for this Solana program.
pub mod escrow { // Starts the Rust module that holds all escrow instruction functions.
    use super::*; // Brings the account structs, enums, and errors from the outer file into this module.

    // Who calls it: the client who wants to create and fund a freelance job.
    // What it does: creates the escrow PDA account, stores job data, and locks SOL inside that PDA.
    // If it fails: no account is created, no SOL is locked, and the transaction is rolled back.
    pub fn create_escrow( // Starts the create_escrow instruction function.
        ctx: Context<CreateEscrow>, // Gives the function access to the accounts required by CreateEscrow.
        job_id: u64, // Receives a unique job number chosen by the frontend or client.
        freelancer: Pubkey, // Receives the freelancer wallet address assigned to this job.
        amount: u64, // Receives the number of lamports to lock; 1 SOL equals 1,000,000,000 lamports.
        deadline: i64, // Receives the Unix timestamp when the job deadline expires.
        description: String, // Receives the human-readable job description.
    ) -> Result<()> { // Returns Ok when the instruction succeeds or an Anchor error when it fails.
        let clock = Clock::get()?; // Reads Solana's current on-chain clock account.
        require!(amount > 0, EscrowError::InvalidAmount); // Rejects jobs that try to lock zero lamports.
        require!(deadline > clock.unix_timestamp, EscrowError::DeadlineInPast); // Rejects deadlines that are not in the future.
        require!(!description.trim().is_empty(), EscrowError::EmptyDescription); // Rejects descriptions that are empty or only spaces.
        require!(description.as_bytes().len() <= EscrowJob::MAX_DESCRIPTION_BYTES, EscrowError::DescriptionTooLong); // Rejects descriptions longer than 200 bytes.

        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the newly created EscrowJob account.
        escrow_job.job_id = job_id; // Stores the unique job identifier inside the account.
        escrow_job.client = ctx.accounts.client.key(); // Stores the wallet address of the client who created the job.
        escrow_job.freelancer = freelancer; // Stores the wallet address of the assigned freelancer.
        escrow_job.amount = amount; // Stores how many lamports are locked for this job.
        escrow_job.deadline = deadline; // Stores the deadline timestamp for refund logic.
        escrow_job.status = EscrowStatus::Open; // Sets the starting status to Open because the freelancer has not accepted yet.
        escrow_job.description = description; // Stores the job description on-chain.
        escrow_job.bump = ctx.bumps.escrow_job; // Stores the PDA bump so the program can find this PDA again later.

        let transfer_accounts = system_program::Transfer { // Creates the account list needed for a system SOL transfer.
            from: ctx.accounts.client.to_account_info(), // Uses the client's wallet as the source of the locked SOL.
            to: ctx.accounts.escrow_job.to_account_info(), // Uses the escrow PDA account as the destination for the locked SOL.
        }; // Ends the transfer account list.
        let transfer_context = CpiContext::new( // Creates a cross-program invocation context for the transfer.
            ctx.accounts.system_program.to_account_info(), // Points the CPI at Solana's system program.
            transfer_accounts, // Passes the source and destination accounts to the system program.
        ); // Ends the CPI context construction.
        system_program::transfer(transfer_context, amount)?; // Moves the client's SOL into the escrow PDA account.

        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the create_escrow instruction.

    // Who calls it: the assigned freelancer.
    // What it does: moves the job from Open to InProgress.
    // If it fails: the job stays Open and no account data changes.
    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> { // Starts the accept_job instruction function.
        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the escrow job account.
        require!(escrow_job.status == EscrowStatus::Open, EscrowError::InvalidStatus); // Allows accepting only when the job is Open.
        escrow_job.status = EscrowStatus::InProgress; // Marks that the freelancer has accepted and started the job.
        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the accept_job instruction.

    // Who calls it: the assigned freelancer.
    // What it does: moves the job from InProgress to Submitted.
    // If it fails: the job stays InProgress and the client cannot approve yet.
    pub fn submit_work(ctx: Context<SubmitWork>) -> Result<()> { // Starts the submit_work instruction function.
        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the escrow job account.
        require!(escrow_job.status == EscrowStatus::InProgress, EscrowError::InvalidStatus); // Allows submission only after the job is InProgress.
        escrow_job.status = EscrowStatus::Submitted; // Marks that the freelancer has submitted the work for review.
        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the submit_work instruction.

    // Who calls it: the client who created and funded the job.
    // What it does: sends the locked SOL to the freelancer and keeps the job saved as history.
    // If it fails: the SOL stays locked and the escrow account remains unchanged.
    pub fn approve_release(ctx: Context<ApproveRelease>) -> Result<()> { // Starts the approve_release instruction function.
        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the escrow job account.
        require!(escrow_job.status == EscrowStatus::Submitted, EscrowError::InvalidStatus); // Allows release only after the freelancer submitted work.

        let amount = escrow_job.amount; // Copies the locked amount before changing balances.
        let escrow_info = escrow_job.to_account_info(); // Converts the escrow account into a low-level account object for lamport movement.
        let freelancer_info = ctx.accounts.freelancer.to_account_info(); // Converts the freelancer wallet into a low-level account object for receiving lamports.
        let escrow_starting_lamports = escrow_info.lamports(); // Reads the escrow account's current lamport balance.
        let freelancer_starting_lamports = freelancer_info.lamports(); // Reads the freelancer wallet's current lamport balance.
        let escrow_ending_lamports = escrow_starting_lamports.checked_sub(amount).ok_or(error!(EscrowError::MathOverflow))?; // Calculates the escrow balance after payment.
        let freelancer_ending_lamports = freelancer_starting_lamports.checked_add(amount).ok_or(error!(EscrowError::MathOverflow))?; // Calculates the freelancer balance after payment.

        **escrow_info.try_borrow_mut_lamports()? = escrow_ending_lamports; // Removes the locked payment amount from the escrow PDA.
        **freelancer_info.try_borrow_mut_lamports()? = freelancer_ending_lamports; // Adds the locked payment amount to the freelancer wallet.
        escrow_job.status = EscrowStatus::Completed; // Marks the job as completed while keeping the account saved on-chain.

        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the approve_release instruction.

    // Who calls it: the client who created and funded the job.
    // What it does: refunds the locked SOL if the deadline passed before submission.
    // If it fails: the SOL stays locked and the escrow account remains unchanged.
    pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> { // Starts the claim_refund instruction function.
        let clock = Clock::get()?; // Reads Solana's current on-chain clock account.
        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the escrow job account.
        let can_refund = escrow_job.status == EscrowStatus::Open || escrow_job.status == EscrowStatus::InProgress; // Checks whether the job is still not submitted.
        require!(can_refund, EscrowError::InvalidStatus); // Allows refund only for Open or InProgress jobs.
        require!(clock.unix_timestamp > escrow_job.deadline, EscrowError::DeadlineNotPassed); // Allows refund only after the deadline has passed.

        let amount = escrow_job.amount; // Copies the locked amount before changing balances.
        let escrow_info = escrow_job.to_account_info(); // Converts the escrow account into a low-level account object for lamport movement.
        let client_info = ctx.accounts.client.to_account_info(); // Converts the client wallet into a low-level account object for receiving the refund.
        let escrow_starting_lamports = escrow_info.lamports(); // Reads the escrow account's current lamport balance.
        let client_starting_lamports = client_info.lamports(); // Reads the client's current lamport balance.
        let escrow_ending_lamports = escrow_starting_lamports.checked_sub(amount).ok_or(error!(EscrowError::MathOverflow))?; // Calculates the escrow balance after refund.
        let client_ending_lamports = client_starting_lamports.checked_add(amount).ok_or(error!(EscrowError::MathOverflow))?; // Calculates the client balance after refund.

        **escrow_info.try_borrow_mut_lamports()? = escrow_ending_lamports; // Removes the locked payment amount from the escrow PDA.
        **client_info.try_borrow_mut_lamports()? = client_ending_lamports; // Adds the locked payment amount back to the client wallet.
        escrow_job.status = EscrowStatus::Refunded; // Marks the job as refunded while keeping the original payment amount for history.

        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the claim_refund instruction.

    // Who calls it: either the client or the assigned freelancer.
    // What it does: moves the job from Submitted to Disputed.
    // If it fails: the job stays Submitted and no dispute flag is saved.
    pub fn raise_dispute(ctx: Context<RaiseDispute>) -> Result<()> { // Starts the raise_dispute instruction function.
        let escrow_job = &mut ctx.accounts.escrow_job; // Gets mutable access to the escrow job account.
        let caller = ctx.accounts.caller.key(); // Reads the wallet address of the person signing this transaction.
        let caller_is_client = caller == escrow_job.client; // Checks whether the signer is the client.
        let caller_is_freelancer = caller == escrow_job.freelancer; // Checks whether the signer is the freelancer.
        require!(caller_is_client || caller_is_freelancer, EscrowError::UnauthorizedDisputer); // Rejects anyone who is not part of this job.
        require!(escrow_job.status == EscrowStatus::Submitted, EscrowError::InvalidStatus); // Allows disputes only after work has been submitted.
        escrow_job.status = EscrowStatus::Disputed; // Marks the job as disputed.
        Ok(()) // Tells Anchor the instruction completed successfully.
    } // Ends the raise_dispute instruction.
} // Ends the escrow instruction module.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the create_escrow instruction.
#[instruction(job_id: u64)] // Lets Anchor use the job_id argument while deriving the escrow PDA address.
pub struct CreateEscrow<'info> { // Defines the accounts needed when a client creates a new escrow job.
    #[account( // Starts Anchor account rules for the escrow_job PDA account.
        init, // Creates this account during the instruction.
        payer = client, // Makes the client pay rent for creating the account.
        space = EscrowJob::SPACE, // Reserves enough bytes to store all EscrowJob data.
        seeds = [b"escrow", client.key().as_ref(), job_id.to_le_bytes().as_ref()], // Derives the PDA from a fixed word, the client wallet, and the job ID.
        bump // Asks Anchor to find the bump seed that makes this PDA valid.
    )] // Ends Anchor account rules for the escrow_job PDA account.
    pub escrow_job: Account<'info, EscrowJob>, // Stores the new on-chain escrow job account.
    #[account(mut)] // Marks the client wallet mutable because it pays rent and sends SOL into escrow.
    pub client: Signer<'info>, // Requires the client wallet to sign the transaction.
    pub system_program: Program<'info, System>, // Provides access to Solana's native program for creating accounts and transferring SOL.
} // Ends the CreateEscrow account context.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the accept_job instruction.
pub struct AcceptJob<'info> { // Defines the accounts needed when a freelancer accepts a job.
    #[account( // Starts Anchor account rules for the escrow_job account.
        mut, // Marks the escrow job mutable because its status will change.
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()], // Rebuilds the PDA address from the stored client and job ID.
        bump = escrow_job.bump, // Uses the stored bump to verify the PDA.
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer // Requires the signer to match the stored freelancer field.
    )] // Ends Anchor account rules for the escrow_job account.
    pub escrow_job: Account<'info, EscrowJob>, // Loads the existing escrow job account.
    pub freelancer: Signer<'info>, // Requires the assigned freelancer wallet to sign the transaction.
} // Ends the AcceptJob account context.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the submit_work instruction.
pub struct SubmitWork<'info> { // Defines the accounts needed when a freelancer submits work.
    #[account( // Starts Anchor account rules for the escrow_job account.
        mut, // Marks the escrow job mutable because its status will change.
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()], // Rebuilds the PDA address from the stored client and job ID.
        bump = escrow_job.bump, // Uses the stored bump to verify the PDA.
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer // Requires the signer to match the stored freelancer field.
    )] // Ends Anchor account rules for the escrow_job account.
    pub escrow_job: Account<'info, EscrowJob>, // Loads the existing escrow job account.
    pub freelancer: Signer<'info>, // Requires the assigned freelancer wallet to sign the transaction.
} // Ends the SubmitWork account context.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the approve_release instruction.
pub struct ApproveRelease<'info> { // Defines the accounts needed when the client releases payment.
    #[account( // Starts Anchor account rules for the escrow_job account.
        mut, // Marks the escrow job mutable because lamports and status will change.
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()], // Rebuilds the PDA address from the stored client and job ID.
        bump = escrow_job.bump, // Uses the stored bump to verify the PDA.
        has_one = client @ EscrowError::UnauthorizedClient, // Requires the signer to match the stored client field.
        has_one = freelancer @ EscrowError::UnauthorizedFreelancer // Requires the payment receiver to match the stored freelancer field.
    )] // Ends Anchor account rules for the escrow_job account.
    pub escrow_job: Account<'info, EscrowJob>, // Loads the existing escrow job account.
    #[account(mut)] // Marks the client mutable because closed-account rent will be returned to this wallet.
    pub client: Signer<'info>, // Requires the client wallet to sign the transaction.
    #[account(mut)] // Marks the freelancer mutable because this wallet receives the locked SOL.
    pub freelancer: SystemAccount<'info>, // Loads the freelancer wallet as a normal system-owned wallet account.
} // Ends the ApproveRelease account context.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the claim_refund instruction.
pub struct ClaimRefund<'info> { // Defines the accounts needed when the client claims a refund.
    #[account( // Starts Anchor account rules for the escrow_job account.
        mut, // Marks the escrow job mutable because lamports, amount, and status will change.
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()], // Rebuilds the PDA address from the stored client and job ID.
        bump = escrow_job.bump, // Uses the stored bump to verify the PDA.
        has_one = client @ EscrowError::UnauthorizedClient // Requires the signer to match the stored client field.
    )] // Ends Anchor account rules for the escrow_job account.
    pub escrow_job: Account<'info, EscrowJob>, // Loads the existing escrow job account.
    #[account(mut)] // Marks the client mutable because this wallet receives the refunded SOL.
    pub client: Signer<'info>, // Requires the client wallet to sign the transaction.
} // Ends the ClaimRefund account context.

#[derive(Accounts)] // Tells Anchor to validate the accounts required by the raise_dispute instruction.
pub struct RaiseDispute<'info> { // Defines the accounts needed when a dispute is raised.
    #[account( // Starts Anchor account rules for the escrow_job account.
        mut, // Marks the escrow job mutable because its status will change.
        seeds = [b"escrow", escrow_job.client.as_ref(), escrow_job.job_id.to_le_bytes().as_ref()], // Rebuilds the PDA address from the stored client and job ID.
        bump = escrow_job.bump // Uses the stored bump to verify the PDA.
    )] // Ends Anchor account rules for the escrow_job account.
    pub escrow_job: Account<'info, EscrowJob>, // Loads the existing escrow job account.
    pub caller: Signer<'info>, // Requires the person raising the dispute to sign the transaction.
} // Ends the RaiseDispute account context.

#[account] // Tells Anchor this struct is an on-chain account type with an 8-byte discriminator.
pub struct EscrowJob { // Defines the data stored for one freelance escrow job.
    pub job_id: u64, // Stores the unique identifier for this job.
    pub client: Pubkey, // Stores the wallet address that created and funded the job.
    pub freelancer: Pubkey, // Stores the wallet address assigned to complete the job.
    pub amount: u64, // Stores the original payment amount in lamports for this job.
    pub deadline: i64, // Stores the Unix timestamp when the client can refund if work was not submitted.
    pub status: EscrowStatus, // Stores the current workflow state of the job.
    pub description: String, // Stores a short plain-text description of the requested work.
    pub bump: u8, // Stores the PDA bump seed, which lets Anchor verify this PDA again in later instructions.
} // Ends the EscrowJob account data struct.

impl EscrowJob { // Starts helper constants for the EscrowJob account.
    pub const MAX_DESCRIPTION_BYTES: usize = 200; // Sets the maximum stored description length to 200 bytes.
    pub const DISCRIMINATOR_BYTES: usize = 8; // Counts Anchor's hidden account type identifier.
    pub const U64_BYTES: usize = 8; // Counts the bytes used by a u64 number.
    pub const PUBKEY_BYTES: usize = 32; // Counts the bytes used by a Solana wallet address.
    pub const I64_BYTES: usize = 8; // Counts the bytes used by an i64 timestamp.
    pub const ENUM_BYTES: usize = 1; // Counts the bytes Anchor needs for this small enum tag.
    pub const STRING_PREFIX_BYTES: usize = 4; // Counts the bytes Anchor stores before a String to record its length.
    pub const U8_BYTES: usize = 1; // Counts the bytes used by a u8 bump.
    pub const SPACE: usize = Self::DISCRIMINATOR_BYTES + Self::U64_BYTES + Self::PUBKEY_BYTES + Self::PUBKEY_BYTES + Self::U64_BYTES + Self::I64_BYTES + Self::ENUM_BYTES + Self::STRING_PREFIX_BYTES + Self::MAX_DESCRIPTION_BYTES + Self::U8_BYTES; // Calculates the total account size needed for rent.
} // Ends helper constants for the EscrowJob account.

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)] // Lets Anchor store, load, copy, and compare this enum.
pub enum EscrowStatus { // Defines every possible state an escrow job can be in.
    Open, // Means the job is funded but the freelancer has not accepted yet.
    InProgress, // Means the freelancer accepted and is working.
    Submitted, // Means the freelancer submitted the work for client review.
    Completed, // Means the client approved and payment was released.
    Disputed, // Means the client or freelancer raised a dispute after submission.
    Refunded, // Means the client claimed the locked SOL back after the deadline.
} // Ends the EscrowStatus enum.

#[error_code] // Tells Anchor to turn this enum into custom program errors.
pub enum EscrowError { // Defines all custom errors this escrow program can return.
    #[msg("The escrow amount must be greater than zero.")] // Gives users a readable message for invalid amounts.
    InvalidAmount, // Means the client tried to create a job with zero lamports.
    #[msg("The deadline must be in the future.")] // Gives users a readable message for past deadlines.
    DeadlineInPast, // Means the client tried to create a job with an expired deadline.
    #[msg("The deadline has not passed yet.")] // Gives users a readable message when refund is too early.
    DeadlineNotPassed, // Means the client tried to refund before the deadline.
    #[msg("The job description cannot be empty.")] // Gives users a readable message for blank descriptions.
    EmptyDescription, // Means the description was empty or only spaces.
    #[msg("The job description cannot be longer than 200 bytes.")] // Gives users a readable message for long descriptions.
    DescriptionTooLong, // Means the description exceeded the allowed size.
    #[msg("Only the client for this escrow job can perform this action.")] // Gives users a readable message for wrong-client calls.
    UnauthorizedClient, // Means a non-client tried to do a client-only action.
    #[msg("Only the assigned freelancer for this escrow job can perform this action.")] // Gives users a readable message for wrong-freelancer calls.
    UnauthorizedFreelancer, // Means a non-freelancer tried to do a freelancer-only action.
    #[msg("Only the client or assigned freelancer can raise a dispute.")] // Gives users a readable message for invalid dispute callers.
    UnauthorizedDisputer, // Means an unrelated wallet tried to dispute the job.
    #[msg("This action is not allowed in the escrow job's current status.")] // Gives users a readable message for wrong workflow state.
    InvalidStatus, // Means the job status does not allow the requested instruction.
    #[msg("A lamport balance calculation failed.")] // Gives users a readable message for safe math failure.
    MathOverflow, // Means a checked add or subtract failed.
} // Ends the EscrowError enum.
