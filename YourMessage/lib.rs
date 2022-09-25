use anchor_lang::prelude::*;

declare_id!("4K8jiCnN3CjSLVyAZqFqxj5b9czFCyHjN27UQp38Tjpe");

#[constant]
pub const USER_DATA:&[u8]=b"user_data";

#[constant]
pub const MESSAGE:&[u8]=b"message";

#[program]
pub mod your_message {
    use super::*;
 
    /*
    This Method Is Used To Create New User Account.  
    @Note:- User Can Create Only One Account From Their Address.
    */
    pub fn create_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        let user_account=&mut ctx.accounts.user_account;
        user_account.authority=ctx.accounts.signer.key();
        user_account.total_messages=0;
        msg!("{} Has Created Their Account {}",ctx.accounts.signer.key(),user_account.key());
        Ok(())
    }

    /*
    This Method Is Used To Create New Message And For Every New Message New Messsage Account Will Be Created.  
    @Note:- User Can Create Multiple Messages And For Each Messasge New Message Account Will Be Created And That Will Be Associated To User Account Address.
    */
    pub fn create_message(ctx: Context<UserMessage>,message:String) -> Result<()> {
        ctx.accounts.user_account.total_messages+=1;
        let message_account=&mut ctx.accounts.message_account;
        message_account.authority=ctx.accounts.signer.key();
        message_account.message=message;
        msg!("{} Has Created Message Account {}",ctx.accounts.signer.key(),message_account.key());
        Ok(())
    }

    /*
    This Method Is Used To Change User Message.
    @Note:- Only Messsage Owner Can Change Their Message From Message Account.
    */
    pub fn change_message(ctx: Context<ChangeMessage>,new_message:String) -> Result<()> {
        let old_message=ctx.accounts.message_account.message.clone();
        ctx.accounts.message_account.message=new_message;
        msg!("Message Changed, Old Message Was '{}'",old_message);
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
#[instruction()]
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

impl Message{
    // It Will Return Space Required For Message Account We Can Also Just Put Space Value Or Calculate Inside "account" Macro, We has Just Created This Method For Increasing Readability.
    fn get_message_size(message:&str) -> usize {
       8 + 32 + 4 + message.len()
       // Account Discriminator = 8 Bytes, Public Key = 32 Bytes And Note That With Every Dynamic Size Data Type Like String And Vector We Have To Add 4.
       // Here We Have One String Type In Struct "Message" Named "message" So We Are Adding Only 4 One Time, If We Have Two String Type And One Vector Type So We Have To Add 4 Three Times For Each.
       // Note That Adding 4 Is Required When We Are Allocating Account Size Same As Data Size Like If Data Size Is 50 Bytes So We Are Allocating Same Size.
    }
}

#[derive(Accounts)]
#[instruction(message:String)]
pub struct UserMessage<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        init,          
        payer = signer,
        space = Message::get_message_size(&message),
        seeds = [MESSAGE,user_account.key().as_ref(),&[user_account.total_messages as u8].as_ref()],
        bump,
        constraint = user_account.authority.key() == signer.key() @Errors::OnlyAuthorised,     
    )]
    pub message_account: Account<'info, Message>,
    pub system_program: Program<'info, System>, 
}

#[derive(Accounts)]
pub struct ChangeMessage<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,has_one=authority @Errors::OnlyAuthorised)]
    pub message_account: Account<'info, Message>,
}

#[error_code]
pub enum Errors {
    #[msg("Only Authorised Address Can Access This Method")]
    OnlyAuthorised,
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
