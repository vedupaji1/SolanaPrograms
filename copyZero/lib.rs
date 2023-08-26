use anchor_lang::prelude::*;

declare_id!("J1JjU3EyDPBaKKpMw8zZoVDnKpEJuquJ8oiW9wAwmGks");

#[program]
pub mod temp11 {
    use super::*;
    // use std::ops::Deref;

    pub fn initialize_data_container(
        ctx: Context<InitDataContainer>,
        temp_data: Vec<u8>,
    ) -> Result<()> {
        ctx.accounts.temp_account.load_init()?.temp_data[..temp_data.len()]
            .copy_from_slice(&temp_data);
        Ok(())
    }
    pub fn push_in_data_container(ctx: Context<StoreTempData>, temp_data: Vec<u8>) -> Result<()> {
        ctx.accounts.temp_account.load_mut()?.temp_data[..temp_data.len()].copy_from_slice(&temp_data);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitDataContainer<'info> {
    #[account(zero)]
    temp_account: AccountLoader<'info, DataContainer>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StoreTempData<'info> {
    #[account(mut)]
    temp_account: AccountLoader<'info, DataContainer>,
}

#[account(zero_copy)]
pub struct DataContainer {
    temp_data: [u8; 50000],
}