use anchor_lang::prelude::*;

declare_id!("Ej57NSJMCKG9ezMzMAVt8TK8hQCeNAuZzrdXAMCWRGRu");

#[program]
pub mod temp {
    use super::*;

    pub fn create_global_account(ctx: Context<CreateGlobalAccount>, data: String) -> Result<()> {
        ctx.accounts.global_stats.authority = ctx.accounts.signer.key();
        ctx.accounts.global_stats.data.push(data);
        msg!(
            "{} Account Created By {}",
            ctx.accounts.global_stats.key(),
            ctx.accounts.signer.key()
        );
        Ok(())
    }

    pub fn store_data(ctx: Context<StoreData>, data: String) -> Result<()> {
        ctx.accounts.global_stats.data.push(data);
        msg!("Data Stored By {}", ctx.accounts.global_stats.key());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(data: String)]
pub struct CreateGlobalAccount<'info> {
    #[account(init,payer=signer,space=GlobalStats::get_space(&data))]
    pub global_stats: Account<'info, GlobalStats>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Default)]
#[account]
pub struct GlobalStats {
    authority: Pubkey,
    data: Vec<String>,
}

impl GlobalStats {
    pub fn get_space(data: &str) -> usize {
        return 8 + std::mem::size_of::<GlobalStats>() + data.len();
    }

    pub fn get_extended_account_space(
        global_stats_account_info: &AccountInfo,
        data: &str,
    ) -> usize {
        return global_stats_account_info.data.as_ref().borrow_mut().len() + 4 + data.len();
    }
}

#[derive(Accounts)]
#[instruction(data: String)]
pub struct StoreData<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
            mut,
            realloc = GlobalStats::get_extended_account_space(&global_stats.to_account_info(),&data),
            realloc::payer = signer,
            realloc::zero = false,
        )]
    pub global_stats: Account<'info, GlobalStats>,
    pub system_program: Program<'info, System>,
}
