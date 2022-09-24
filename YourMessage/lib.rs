use anchor_lang::prelude::*;

declare_id!("4K8jiCnN3CjSLVyAZqFqxj5b9czFCyHjN27UQp38Tjpe");

#[constant]
pub const USER_DATA:&[u8]=b"Op";

#[constant]
pub const MESSAGE:&[u8]=b"message";

#[program]
pub mod your_message {
    use super::*;

    pub fn create_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        let user_account=&mut ctx.accounts.user_account;
        user_account.authority=ctx.accounts.signer.key();
        user_account.total_messages=0;
        Ok(())
    }

    pub fn create_message(ctx: Context<UserMessage>,message:String) -> Result<()> {
        ctx.accounts.user_account.total_messages+=1;
        let message_account=&mut ctx.accounts.message_account;
        message_account.authority=ctx.accounts.signer.key();
        message_account.message=message;
        Ok(())
    }
    
}

#[account]
#[derive(Default)]
pub struct UserAccount {
    authority:Pubkey,
    total_messages:u32,
}

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + std::mem::size_of::<UserAccount>(),
        seeds = [USER_DATA, signer.key().as_ref()], 
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct Message {
    authority:Pubkey,
    message:String,
}

#[derive(Accounts)]
pub struct UserMessage<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        init,          
        payer = signer,
        space = 8 + 1000,
        seeds = [MESSAGE,user_account.key().as_ref(),&[user_account.total_messages as u8].as_ref()],
        bump,
        constraint = user_account.authority.key() == signer.key(),     
    )]
    pub message_account: Account<'info, Message>,
    pub system_program: Program<'info, System>, 
}

/* 

Structure Of Accounts:-

          UserAddress  // User Wallet Address
              |
              |
              V
     UserAccountInContract // Using "create_account()" Method User Account Will Be Created 
             /|\
            / | \
           /  |  \
          /   |   \
         /    |    \
        /     |     \
   MsgAc1   MsgAc2  MsgAcN // Using "create_message()" Message Account Will Be Created And "total_messages" Value Of User Account Will Be Increamented

   Note:-
   MsgAc = MessageAccount
   
*/
