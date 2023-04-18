use anchor_lang::prelude::*;
use anchor_spl::token;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[constant]
pub const PROGRAM_STATS: &[u8] = b"programStats";

#[constant]
pub const USER_STATS: &[u8] = b"userStats";

#[constant]
pub const SWAP_STATS: &[u8] = b"swapStats";

#[constant]
pub const ESCROW_TOKEN_ACCOUNT: &[u8] = b"escrowTokenAccount";

#[program]

pub mod escrow_swap {
    use super::*;

    pub fn create_program_stats(ctx: Context<CreateProgramStats>) -> Result<()> {
        let program_stats = &mut ctx.accounts.program_stats;
        program_stats.authority = ctx.accounts.signer.key();
        program_stats.is_swap_in_hold = false;
        //program_stats.fee_amount = 1.0;
        program_stats.total_swap_created = 0;
        program_stats.bump = *ctx.bumps.get("program_stats").unwrap();
        msg!(
            "Program Stats Account {} Created By {}",
            program_stats.key(),
            ctx.accounts.signer.key()
        );
        Ok(())
    }

    pub fn create_user_stats(ctx: Context<CreateUserStats>) -> Result<()> {
        ctx.accounts.user_stats.authority = ctx.accounts.signer.key();
        Ok(())
    }

    pub fn create_swap(
        ctx: Context<CreateSwap>,
        token_b_address: Pubkey,
        token_a_amount: u64,
        token_b_amount: u64,
        user_a_token_b_account_address: Pubkey,
        user_b_address: Pubkey,
    ) -> Result<()> {
        require!(
            ctx.accounts.program_stats.is_swap_in_hold == false,
            Errors::ProgramIsOnHold
        );
        require!(
            ctx.accounts.user_a_token_a_account.amount >= token_a_amount,
            Errors::InsufficientBalance
        );
        let swap_escrow_stats = &mut ctx.accounts.swap_escrow_stats;
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_a_token_a_account.to_account_info(),
                    to: ctx.accounts.escrow_token_a_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            token_a_amount,
        )?;

        msg!(
            "{} Amount Of {} Token transferred To {} Account Of SwapEscrow Account",
            token_a_amount,
            ctx.accounts.token_a_account.key(),
            ctx.accounts.escrow_token_a_account.key()
        );

        swap_escrow_stats.id = ctx.accounts.program_stats.total_swap_created;
        ctx.accounts.program_stats.total_swap_created += 1;
        swap_escrow_stats.user_a_address = ctx.accounts.signer.key();
        swap_escrow_stats.user_b_address = user_b_address;
        swap_escrow_stats.token_a_address = ctx.accounts.token_a_account.key();
        swap_escrow_stats.token_b_address = token_b_address;
        swap_escrow_stats.token_a_amount = token_a_amount;
        swap_escrow_stats.token_b_amount = token_b_amount;
        swap_escrow_stats.user_a_token_b_account_address = user_a_token_b_account_address;
        swap_escrow_stats.swap_status = SwapStatus::UserADepositedTokens;

        ctx.accounts
            .user_a_stats
            .swaps_created
            .push(ctx.accounts.swap_escrow_stats.key());
        Ok(())
    }

    pub fn swap_tokens(ctx: Context<SwapTokens>) -> Result<()> {
        require!(
            ctx.accounts.program_stats.is_swap_in_hold == false,
            Errors::ProgramIsOnHold
        );
        require!(
            ctx.accounts.swap_escrow_stats.swap_status == SwapStatus::UserADepositedTokens,
            Errors::NoStatusForSwap
        );
        require!(
            ctx.accounts.token_a_account.key() == ctx.accounts.swap_escrow_stats.token_a_address
                && ctx.accounts.token_b_account.key()
                    == ctx.accounts.swap_escrow_stats.token_b_address,
            Errors::InvalidTokenAccount
        );
        require!(
            ctx.accounts.user_b_token_b_account.amount
                >= ctx.accounts.swap_escrow_stats.token_b_amount,
            Errors::InsufficientBalance
        );

        ctx.accounts.swap_escrow_stats.swap_status = SwapStatus::SwapCompleted;

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_b_token_b_account.to_account_info(),
                    to: ctx.accounts.user_a_token_b_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            ctx.accounts.swap_escrow_stats.token_b_amount,
        )?;
        msg!(
            "{} Token A Transferred To {} From {}",
            ctx.accounts.swap_escrow_stats.token_b_amount,
            ctx.accounts.swap_escrow_stats.user_a_address,
            ctx.accounts.swap_escrow_stats.user_b_address
        );
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.escrow_token_a_account.to_account_info(),
                    to: ctx.accounts.user_b_token_a_account.to_account_info(),
                    authority: ctx.accounts.program_stats.to_account_info(),
                },
                &[&[PROGRAM_STATS, &[ctx.accounts.program_stats.bump]]],
            ),
            ctx.accounts.swap_escrow_stats.token_a_amount,
        )?;
        msg!(
            "{} Token A Transferred To {} From {}",
            ctx.accounts.swap_escrow_stats.token_a_amount,
            ctx.accounts.swap_escrow_stats.user_b_address,
            ctx.accounts.swap_escrow_stats.user_a_address
        );
        Ok(())
    }
    pub fn cancel_swap(ctx: Context<CancelSwap>) -> Result<()> {
        require!(
            ctx.accounts.swap_escrow_stats.user_a_address == ctx.accounts.signer.key(),
            Errors::OnlyUserACanAccessThisMethod
        );
        require!(
            ctx.accounts.swap_escrow_stats.swap_status == SwapStatus::UserADepositedTokens,
            Errors::SwapCannotBeCancelled
        );
        ctx.accounts.swap_escrow_stats.swap_status = SwapStatus::SwapCancelled;
        msg!(
            "Swap {} Is Cancelled By {}",
            ctx.accounts.swap_escrow_stats.key(),
            ctx.accounts.signer.key()
        );
        Ok(())
    }

    pub fn hold_escrow_swap(ctx: Context<HoldEscrowSwap>) -> Result<()> {
        ctx.accounts.program_stats.is_swap_in_hold = true;
        msg!("EscrowSwap Is In Hold By {}", ctx.accounts.authority.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateProgramStats<'info> {
    #[account(init,payer=signer,space=8+std::mem::size_of::<ProgramStats>(),seeds=[PROGRAM_STATS],bump)]
    program_stats: Account<'info, ProgramStats>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Debug)]
#[account]
pub struct ProgramStats {
    authority: Pubkey,
    bump: u8,
    is_swap_in_hold: bool,
    //fee_amount: f32,
    total_swap_created: u64,
}

#[derive(Accounts)]
pub struct CreateUserStats<'info> {
    #[account(init,payer=signer,space=UserStats::user_stats_space(),seeds=[USER_STATS,signer.key().as_ref()],bump)]
    user_stats: Account<'info, UserStats>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Debug)]
#[account]
pub struct UserStats {
    authority: Pubkey,
    swaps_created: Vec<Pubkey>,
    swaps_joined: Vec<Pubkey>,
}

impl UserStats {
    fn user_stats_space() -> usize {
        return 8 + 32 + 4 + 1000;
    }
}

#[derive(Accounts)]
#[instruction(bump:u8)]
pub struct CreateSwap<'info> {
    #[account(init,payer=signer,space=8+std::mem::size_of::<SwapEscrow>(),seeds=[SWAP_STATS,&[program_stats.total_swap_created as u8].as_ref()],bump)]
    swap_escrow_stats: Box<Account<'info, SwapEscrow>>,
    #[account(init_if_needed, payer = signer, seeds=[ESCROW_TOKEN_ACCOUNT,token_a_account.key().as_ref()] ,bump,token::mint = token_a_account, token::authority = program_stats)]
    escrow_token_a_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    program_stats: Box<Account<'info, ProgramStats>>,
    #[account(mut)]
    user_a_stats: Box<Account<'info, UserStats>>,
    #[account(mut)]
    user_a_token_a_account: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    token_a_account: AccountInfo<'info>,
    #[account(mut)]
    signer: Signer<'info>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, token::Token>,
    system_program: Program<'info, System>,
}

#[derive(Debug)]
#[account]
pub struct SwapEscrow {
    id: u64,
    user_a_address: Pubkey,
    user_b_address: Pubkey,
    token_a_address: Pubkey,
    token_b_address: Pubkey,
    token_a_amount: u64,
    token_b_amount: u64,
    user_a_token_b_account_address: Pubkey,
    user_b_token_a_account_address: Pubkey,
    swap_status: SwapStatus,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
enum SwapStatus {
    UserADepositedTokens,
    UserBDepositedTokens,
    SwapCompleted,
    SwapCancelled,
}

#[derive(Accounts)]
pub struct SwapTokens<'info> {
    swap_escrow_stats: Box<Account<'info, SwapEscrow>>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    escrow_token_a_account: AccountInfo<'info>,
    program_stats: Box<Account<'info, ProgramStats>>,
    #[account(mut)]
    user_b_stats: Box<Account<'info, UserStats>>,
    #[account(mut)]
    user_a_token_b_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    user_b_token_a_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    user_b_token_b_account: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    token_a_account: AccountInfo<'info>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    token_b_account: AccountInfo<'info>,
    #[account(mut)]
    signer: Signer<'info>,
    token_program: Program<'info, token::Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelSwap<'info> {
    #[account(mut)]
    swap_escrow_stats: Account<'info, SwapEscrow>,
    signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct HoldEscrowSwap<'info> {
    #[account(mut,has_one=authority)]
    program_stats: Account<'info, ProgramStats>,
    authority: Signer<'info>,
}
#[error_code]
pub enum Errors {
    #[msg("EscrowSwap Program Is Now On Hold")]
    ProgramIsOnHold,

    #[msg("Insufficient Token Balance")]
    InsufficientBalance,

    #[msg("Token A Or Token B Account Is Invalid")]
    InvalidTokenAccount,

    #[msg("Sending Amount Is Not Matching To Expected Receiving Amount")]
    InvalidTokenBAmount,

    #[msg("There Is Not Any Status For Token Swap")]
    NoStatusForSwap,

    #[msg("Only User A Or Swap Creator Can Access This Method")]
    OnlyUserACanAccessThisMethod,

    #[msg("Swap Cannot Be Cancelled, Swap Can Be Cancelled Only When Swap Is In Deposited Status")]
    SwapCannotBeCancelled,
}
