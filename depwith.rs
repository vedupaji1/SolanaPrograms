use anchor_lang::prelude::*;

declare_id!("FxxyaoKKNhMHsowmn23mujx2biiqDsxTRRkTcdJ3vLy6");

#[program]
pub mod game {
    use super::*;

    pub fn create_stats(ctx: Context<CreateStats>) -> Result<()> {
        // Here We Are Storing Bump Value In Stats Account So We Can Use That Bump Value
        ctx.accounts.stats.bump = *ctx.bumps.get("stats").unwrap();
        Ok(())
    }

    pub fn deposite_sol(ctx: Context<DepAndWith>, amount: u64) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.stats.key(),
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.stats.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn withdraw_sol(ctx: Context<DepAndWith>, amount: u64) -> Result<()> {
        **ctx
            .accounts
            .stats
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[account]
pub struct Stats {
    bump: u8,
}

// validation struct
#[derive(Accounts)]
pub struct CreateStats<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 1000, seeds = [b"stats"], bump
    )]
    pub stats: Account<'info, Stats>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepAndWith<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub stats: Account<'info, Stats>,
    pub system_program: Program<'info, System>,
}
