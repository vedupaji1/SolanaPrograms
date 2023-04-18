use anchor_lang::prelude::*;
declare_id!("7v8aJ3fSUb2SYbZH2bBse8BGmdjZBZykbkWnVuvo3Uvd");
#[constant]
pub const USER_ACCOUNT: &[u8] = b"userAccount";
#[constant]
pub const SHART_ACCOUNT: &[u8] = b"shartAccount";
#[program]
pub mod shart_the_game {
    use super::*;
    pub fn create_user_account(ctx: Context<UserAccount>) -> Result<()> {
        ctx.accounts.user_account.authority = ctx.accounts.signer.key();
        ctx.accounts.user_account.total_sharts_created = 0;
        msg!(
            "User {} Created An Account {}",
            ctx.accounts.signer.key(),
            ctx.accounts.user_account.key()
        );
        Ok(())
    }
    // Here We Have To Use Vector For Participants,
    // SolPg IDE Does Not Supports Vector Input And Anchor Does Not Supports Dynamic Size Array Thats Why We Are Using Fixed Size Array
    pub fn create_shart(
        ctx: Context<ShartAccount>,
        can_participate: [Pubkey; 2],
        maker_shares: u8,
        amount: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.to_account_info().lamports() >= amount,
            Errors::InsufficientBalance
        );
        require!(start_time < end_time, Errors::StartTimeIsGreaterThanEndTime);
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        require!(
            current_time < end_time && current_time <= start_time,
            Errors::TimeIsGreterThanCurTime
        );
        require!(maker_shares < 90, Errors::InvalidMakerShareAmount);
        let shart_account = &mut ctx.accounts.shart_account;
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &shart_account.key(),
            amount,
        );
        let res = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                shart_account.to_account_info(),
            ],
        );
        msg!("{:?}", res);
        shart_account.maker = ctx.accounts.authority.key();
        shart_account.maker_shares = maker_shares;
        shart_account.amount = amount;
        shart_account.start_time = start_time;
        shart_account.end_time = end_time;
        shart_account.is_withdrawed = false;
        for i in 0..can_participate.len() {
            if can_participate[i] == ctx.accounts.authority.key() {
                return err!(Errors::MakerPubkeyInArray);
            }
        }
        for i in 0..can_participate.len() {
            for j in i..can_participate.len() - 1 {
                if can_participate[i] == can_participate[j + 1] {
                    return err!(Errors::DuplicatePubkeyInArray);
                }
            }
            shart_account.can_participate.push(can_participate[i]);
        }
        ctx.accounts.user_account.total_sharts_created += 1;
        msg!(
            "Shart Account {} Created By {}",
            shart_account.key(),
            ctx.accounts.authority.key()
        );
        Ok(())
    }
    pub fn join_shart(ctx: Context<JoinShart>) -> Result<()> {
        require!(
            ctx.accounts
                .shart_account
                .can_participate
                .iter()
                .position(|&x| x == ctx.accounts.authority.key())
                .unwrap_or_else(|| 0)
                != 0,
            Errors::InvalidParticipant
        );
        //Here We Can Also Check That Whether Signer Has Already Participated Or By Checking Their Address In ctx.accounts.shart_account.participants
        require!(
            ctx.accounts
                .user_account
                .joined_sharts
                .iter()
                .position(|&x| x == ctx.accounts.shart_account.key())
                .unwrap_or_else(|| 0)
                == 0,
            Errors::AlreadyParticipated
        );
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        let shart_account = &mut ctx.accounts.shart_account;
        require!(
            current_time >= shart_account.start_time,
            Errors::ShartNotStarted
        );
        require!(
            current_time <= shart_account.end_time,
            Errors::ShartTimeOver
        );
        require!(
            ctx.accounts.authority.to_account_info().lamports() >= shart_account.amount,
            Errors::InsufficientBalance
        );
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &shart_account.key(),
            shart_account.amount,
        );
        let res = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                shart_account.to_account_info(),
            ],
        );
        msg!("{:?}", res);
        ctx.accounts
            .user_account
            .joined_sharts
            .push(shart_account.key());

        shart_account
            .participants
            .push(ctx.accounts.authority.key());
        msg!(
            "{:?} Has Joined {:?} Shart",
            ctx.accounts.authority.key(),
            shart_account.key()
        );
        Ok(())
    }
    pub fn declare_winner(ctx: Context<DeclareWinner>) -> Result<()> {
        require!(
            ctx.accounts.shart_account.winner == Pubkey::default(),
            Errors::WinnerAlreadyDeclared
        );
        require!(
            Clock::get().unwrap().unix_timestamp as u64 >= ctx.accounts.shart_account.end_time,
            Errors::ShartIsLive
        );

        let total_participants = ctx.accounts.shart_account.participants.len() as i64;
        let winner: Pubkey;

        let rand_number =
            (Clock::get().unwrap().unix_timestamp % (total_participants + 1)) as usize;
        if rand_number == ctx.accounts.shart_account.participants.len() {
            winner = ctx.accounts.shart_account.maker;
        } else {
            winner = ctx.accounts.shart_account.participants[rand_number];
        }
        ctx.accounts.shart_account.winner = winner;
        msg!(
            "Winner Of Shart {:?} Is {:?}",
            ctx.accounts.shart_account.key(),
            winner
        );
        Ok(())
    }

    pub fn withdraw_winning_amount(ctx: Context<WithdrawAmount>) -> Result<()> {
        require!(
            ctx.accounts.shart_account.is_withdrawed == false,
            Errors::RewardWithdrawed
        );
        ctx.accounts.shart_account.is_withdrawed = true;
        let shart_account = &mut ctx.accounts.shart_account;
        let total_amount = shart_account.amount * (shart_account.participants.len() as u64 + 1);
        let maker_amount = (total_amount * shart_account.maker_shares as u64) / 100;
        let winner_amount = total_amount - maker_amount;
        **shart_account.to_account_info().try_borrow_mut_lamports()? -= 1000;
        **ctx.accounts.winner.try_borrow_mut_lamports()? += 1000;
        msg!(
            "{:?} Winning Amount Transfer To {:?}",
            winner_amount,
            ctx.accounts.winner.key()
        );
        **shart_account.to_account_info().try_borrow_mut_lamports()? -= maker_amount;
        **ctx.accounts.maker.try_borrow_mut_lamports()? += maker_amount;
        msg!(
            "{:?} Winning Amount Transfer To {:?}",
            maker_amount,
            ctx.accounts.maker.key()
        );
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct User {
    authority: Pubkey,
    total_sharts_created: u32,
    joined_sharts: Vec<Pubkey>,
}
#[derive(Accounts)]
pub struct UserAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer=signer,
        space=8+32+4+1000,
        seeds=[USER_ACCOUNT,signer.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, User>,
    pub system_program: Program<'info, System>,
}
#[account]
#[derive(Default)]
pub struct Shart {
    maker: Pubkey,
    maker_shares: u8,
    can_participate: Vec<Pubkey>,
    participants: Vec<Pubkey>,
    amount: u64,
    start_time: u64,
    end_time: u64,
    winner: Pubkey,
    is_withdrawed: bool,
}
impl Shart {
    pub fn calculate_space() -> usize {
        return 8 + 32 + 1 + 4 + 1000 + 4 + 1000 + 8 + 8 + 8 + 32 + 1;
    }
}
#[derive(Accounts)]
pub struct ShartAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,has_one=authority @Errors::UnauthorizedUser)]
    pub user_account: Account<'info, User>,
    #[account(
        init,
        payer=authority,
        space=Shart::calculate_space(),
        seeds=[
            SHART_ACCOUNT,
            authority.key().as_ref(),
            &[user_account.total_sharts_created as u8].as_ref()
        ],
        bump,
    )]
    pub shart_account: Account<'info, Shart>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct JoinShart<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,has_one=authority @Errors::UnauthorizedUser)]
    pub user_account: Account<'info, User>,
    #[account(mut,constraint = shart_account.maker != authority.key() @Errors::MakerCannotJoinShart)]
    pub shart_account: Account<'info, Shart>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct DeclareWinner<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub shart_account: Account<'info, Shart>,
}

#[derive(Accounts)]
pub struct WithdrawAmount<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,
    #[account(mut)]
    pub maker: AccountInfo<'info>,
    #[account(mut,has_one=winner,has_one=maker)]
    pub shart_account: Account<'info, Shart>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum Errors {
    #[msg("You Don't Have An Authority To Modify This Account")]
    UnauthorizedUser,
    #[msg("You Don't Have Enough Balance")]
    InsufficientBalance,
    #[msg("End Time Must Be Greater Than Start Time")]
    StartTimeIsGreaterThanEndTime,
    #[msg("End Time Should Smaller Than Current Time")]
    TimeIsGreterThanCurTime,
    #[msg("Maker Shares Cannot Be More Than 90%")]
    InvalidMakerShareAmount,
    #[msg("There Should Not Maker Public Key In Array Input")]
    MakerPubkeyInArray,
    #[msg("There Are Duplicate Public Keys Array Input")]
    DuplicatePubkeyInArray,
    #[msg("You Has Already Participated In Shart")]
    AlreadyParticipated,
    #[msg("You Cannot Join This Shart")]
    InvalidParticipant,
    #[msg("Maker Cannot Join Any Shart")]
    MakerCannotJoinShart,
    #[msg("Shart Is Not Sharted Yet")]
    ShartNotStarted,
    #[msg("Shart Time Is Over, Now You Cannot Join Shart")]
    ShartTimeOver,
    #[msg("Shart Is Still Live")]
    ShartIsLive,
    #[msg("Winner Is Already Ended, Now You Cannot Declare Shart Winner")]
    WinnerAlreadyDeclared,
    #[msg("Rewards Are Already Withdrawed")]
    RewardWithdrawed,
}
