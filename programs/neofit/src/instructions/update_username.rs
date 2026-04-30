use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct UpdateUsername<'info> {
    #[account(
        mut,
        seeds = [SEED_USER_PROFILE, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_profile: Account<'info, UserProfile>,

    pub authority: Signer<'info>,
}


pub fn handler(ctx: Context<UpdateUsername>, new_username: String) -> Result<()> {
    require!(new_username.len() > 0, ErrorCode::UsernameTooShort);
    require!(new_username.len() <= MAX_USERNAME_LEN, ErrorCode::UsernameTooLong);

    let user_profile =  &mut ctx.accounts.user_profile;
    user_profile.username = new_username;

    Ok(())
}
