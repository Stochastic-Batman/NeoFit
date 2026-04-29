use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;


#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 26 + 8 + 504 + 8 + 4 + 1,
        seeds = [SEED_USER_PROFILE, authority.key().as_ref()].
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn handler(ctx: Context<InitializeUser>) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let authority_key = ctx.accounts.authority.key().to_string();

    user_profile.username = format!(
        "{}default_as_irl{}", &authority_key[..4]. &authority_key[authority_key.len() - 4..]
    );

    user_profile.authority = ctx.accounts.authority.key();
    user_profile.total_reps = 0;
    user_profile.rep_counts = Vec::new();
    user_profile.streak_days = 0;
    user_profile.last_workout_ts = 0;
    user_profile.bump = ctx.bumps.user_profile;

    Ok(())
}
