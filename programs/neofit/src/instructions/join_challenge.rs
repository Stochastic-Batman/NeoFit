use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct JoinChallenge<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 79 + 1 + 1 + 1,
        seeds = [SEED_ENROLLMENT, challenge.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub enrollment: Account<'info, Enrollment>,

    #[account(mut)]
    pub challenge: Account<'info, Challenge>,

    #[account(
        seeds = [SEED_USER_PROFILE, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn handler(ctx: Context<JoinChallenge>) -> Result<()> {
    let challenge = &mut ctx.accounts.challenge;
    let enrollment = &mut ctx.accounts.enrollment;

    require!(challenge.is_active, ErrorCode::ChallengeInactive);
    require!(Clock::get()?.unix_timestamp < challenge.deadline_ts, ErrorCode::ChallengeExpired);

    if challenge.entry_fee_lamports > 0 {
        let fee = challenge.entry_fee_lamports;
        challenge.pool_lamports = challenge.pool_lamports
            .checked_add(fee)
            .ok_or(ErrorCode::Overflow)?;
    }

    enrollment.user = ctx.accounts.authority.key();
    enrollment.challenge = challenge.key();

    let mut initial_reps = Vec::new();
    for req in challenge.requirements.iter() {
        initial_reps.push(ExerciseCount { exercise_id: req.exercise_id, count: 0 });
    }

    enrollment.reps_logged = initial_reps;
    enrollment.completed = false;
    enrollment.reward_claimed = false;
    enrollment.bump = ctx.bumps.enrollment;

    Ok(())
}
