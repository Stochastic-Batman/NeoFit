use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        seeds = [SEED_ENROLLMENT, challenge.key().as_ref(), authority.key().as_ref()],
        bump = enrollment.bump,
        constraint = enrollment.user == authority.key(),
        constraint = enrollment.challenge == challenge.key()
    )]
    pub enrollment: Account<'info, Enrollment>,

    #[account(mut)]
    pub challenge: Account<'info, Challenge>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn handler(ctx: Context<ClaimReward>) -> Result<()> {
    let enrollment = &mut ctx.accounts.enrollment;
    let challenge = &mut ctx.accounts.challenge;

    require!(enrollment.completed, ErrorCode::NotAuthorized);
    require!(!enrollment.reward_claimed, ErrorCode::AlreadyClaimed);

    if challenge.completers > 0 && challenge.pool_lamports > 0 {
        let fee_multiplier = 10_000u64.checked_sub(PROTOCOL_FEE_BPS).ok_or(ErrorCode::Overflow)?;
        let net_pool = challenge.pool_lamports.checked_mul(fee_multiplier).ok_or(ErrorCode::Overflow)? / 10_000;
        let user_share = net_pool.checked_div(challenge.completers as u64).ok_or(ErrorCode::Overflow)?;

        let seeds = &[
            SEED_CHALLENGE,
            challenge.authority.as_ref(),
            &challenge.nonce.to_le_bytes(),
            &[challenge.bump],
        ];
        #[allow(unused_variables)]
        let signer = &[&seeds[..]];
        
        challenge.sub_lamports(user_share)?;
        ctx.accounts.authority.add_lamports(user_share)?;
    }

    enrollment.reward_claimed = true;

    Ok(())
}
