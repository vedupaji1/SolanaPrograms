use anchor_lang::prelude::*;

declare_id!("13gedgAiUg9Vow8GxsMHMW6fo9vATcruCS3WAcCiCJE");

#[program]
pub mod temp_op {
    use super::*;

    pub fn create_state(ctx: Context<ProgramState>, max_participants: u32) -> Result<()> {
        ctx.accounts.state_data.owner = ctx.accounts.signer.key();
        ctx.accounts.state_data.max_participants = max_participants;
        Ok(())
    }

    pub fn create_voter(ctx: Context<UserAccount>) -> Result<()> {
        ctx.accounts.voter_data.creator = ctx.accounts.signer.key();
        Ok(())
    }

    pub fn create_voting(
        ctx: Context<VotingSystem>,
        new_voting_address: Pubkey,
        start_time: i64,
        end_time: i64,
        participants: Vec<Pubkey>,
    ) -> Result<()> {
        require!(start_time < end_time, Error::StartTimeIsGreater);
        require!(
            start_time > Clock::get().unwrap().unix_timestamp,
            Error::GreaterThanCurrentTime
        );
        let total_participants = participants.len() as u32;
        require!(
            total_participants <= ctx.accounts.state_data.max_participants,
            Error::MaxParticipantsExceeds
        );

        let mut total_keys = 0;
        for i in 0..participants.len() {
            for j in 0..participants.len() {
                if participants[i] == participants[j] {
                    if total_keys == 1 {
                        return err!(Error::DuplicateParticipantAddress);
                    } else {
                        total_keys = 1;
                    }
                }
            }
            total_keys = 0;
        }

        ctx.accounts.voting_data.creator = ctx.accounts.signer.key();
        ctx.accounts.voting_data.start_time = start_time;
        ctx.accounts.voting_data.end_time = end_time;
        ctx.accounts.voting_data.participants = participants;
        ctx.accounts
            .state_data
            .voting_systems_addresses
            .push(new_voting_address);

        Ok(())
    }

    pub fn give_vote(ctx: Context<GiveVote>, participant_address_index: i32) -> Result<()> {
        require!(
            ctx.accounts.voting_data.start_time <= Clock::get().unwrap().unix_timestamp,
            Error::VotingNotStarted,
        );
        require!(
            ctx.accounts.voting_data.end_time > Clock::get().unwrap().unix_timestamp,
            Error::VotingEnded
        );
        let voting_address_index = ctx
            .accounts
            .voter_data
            .voted_in
            .iter()
            .position(|&x| x == ctx.accounts.voting_data.key())
            .unwrap_or(0);
        require!(voting_address_index == 0, Error::AlreadyVoted);

        ctx.accounts
            .voter_data
            .voted_in
            .push(ctx.accounts.voting_data.key());

        ctx.accounts.voting_data.participants_votes[participant_address_index as usize] += 1;

        Ok(())
    }

    pub fn declare_winner(ctx: Context<DeclareWinner>) -> Result<()> {
        require!(
            ctx.accounts.voting_data.creator == ctx.accounts.signer.key(),
            Error::OnlyOwner
        );
        require!(
            ctx.accounts.voting_data.start_time <= Clock::get().unwrap().unix_timestamp,
            Error::VotingNotStarted
        );
        require!(
            ctx.accounts.voting_data.end_time < Clock::get().unwrap().unix_timestamp,
            Error::VotingNotEnded
        );
        match get_winner(&ctx.accounts.voting_data.participants_votes) {
            Some(winner_address_index) => {
                ctx.accounts.voting_data.winner =
                    ctx.accounts.voting_data.participants[winner_address_index];
            }
            None => {
                return err!(Error::NoOneIsWinner);
            }
        }
        Ok(())
    }

    pub fn set_max_participants(
        ctx: Context<ModifyState>,
        new_max_participants_number: u32,
    ) -> Result<()> {
        require!(
            ctx.accounts.signer.key() == ctx.accounts.state_data.owner.key(),
            Error::OnlyOwner
        );

        ctx.accounts.state_data.max_participants = new_max_participants_number;
        msg!(
            "Max Participant Limit Modified, New Max Participant Limit Is {:?}",
            new_max_participants_number
        );

        Ok(())
    }
}

fn get_winner(participants: &Vec<i32>) -> Option<usize> {
    let mut winner_address_index: usize = 0;
    let mut i = 0;
    for total_votes in participants {
        if participants[winner_address_index] < total_votes.clone() {
            winner_address_index = i;
            i += 1;
        }
    }
    Some(winner_address_index)
}

#[derive(Accounts)]
pub struct ProgramState<'info> {
    #[account(init,payer=signer,space=8+32+320)]
    pub state_data: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ModifyState<'info> {
    #[account(mut)]
    pub state_data: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub voting_systems_addresses: Vec<Pubkey>,
    pub max_participants: u32,
}

#[derive(Accounts)]
pub struct VotingSystem<'info> {
    #[account(init,payer=signer,space=8+32+320)]
    pub voting_data: Account<'info, Voting>,
    #[account(mut)]
    pub state_data: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct GiveVote<'info> {
    #[account(mut)]
    pub voter_data: Account<'info, Voter>,
    #[account(mut)]
    pub voting_data: Account<'info, Voting>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct DeclareWinner<'info> {
    #[account(mut)]
    pub voting_data: Account<'info, Voting>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[account]
pub struct Voting {
    pub creator: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub participants: Vec<Pubkey>,
    pub participants_votes: Vec<i32>,
    pub winner: Pubkey,
}

#[derive(Accounts)]
pub struct UserAccount<'info> {
    #[account(init,payer=signer,space=8+32+320)]
    pub voter_data: Account<'info, Voter>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[account]
pub struct Voter {
    creator: Pubkey,
    voted_in: Vec<Pubkey>,
}

#[error_code]
pub enum Error {
    #[msg("Only Owner Can Access This Method")]
    OnlyOwner,

    #[msg("Start Time And End Time Should Greater Than Current Time")]
    GreaterThanCurrentTime,

    #[msg("Start Time Should Lesser Than End Time")]
    StartTimeIsGreater,

    #[msg("Maximum Participants Exceeds")]
    MaxParticipantsExceeds,

    #[msg("All Participants Addresses Should Unique")]
    ParticipantAddressRepeated,

    #[msg("Participant Addresses Should Not Duplicate")]
    DuplicateParticipantAddress,

    #[msg("Voting Yet Not Started")]
    VotingNotStarted,

    #[msg("Voting Is Ended")]
    VotingEnded,

    #[msg("You Has Already Voted In This Voting")]
    AlreadyVoted,

    #[msg("Winner Cannot Be Declare Before Voting End")]
    VotingNotEnded,

    #[msg("Winner Cannot Be Declare Before Voting End")]
    NoOneIsWinner,
}
