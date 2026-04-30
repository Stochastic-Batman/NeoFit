use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(title: String, requirements: Vec<ExerciseRequirement>, entry_fee: u64, deadline: i64, nonce: u64)]
pub struct CreateChallenge<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 36 + 49 + 8 + 8 + 4 + 8 + 1 + 8 + 1,
        seeds = [SEED_CHALLENGE, authority.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub challenge: Account<'info, Challenge>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateChallenge>, 
    title: String, 
    requirements: Vec<ExerciseRequirement>, 
    entry_fee: u64, 
    deadline: i64, 
    nonce: u64
) -> Result<()> {
    
    let current_time = Clock::get()?.unix_timestamp;
    require!(deadline > current_time, ErrorCode::ChallengeExpired); 

    require!(title.len() <= 22, ErrorCode::UsernameTooLong);
    require!(requirements.len() >= 1 && requirements.len() <= MAX_REQUIREMENTS, ErrorCode::TooManyRequirements);

    for req in requirements.iter() {
        require!(req.exercise_id <= MAX_EXERCISE_ID, ErrorCode::InvalidExerciseId);
        require!(req.rep_target > 0, ErrorCode::InvalidExerciseId); 
    }

    let challenge = &mut ctx.accounts.challenge;
    challenge.authority = ctx.accounts.authority.key();
    challenge.title = title;
    challenge.requirements = requirements;
    challenge.entry_fee_lamports = entry_fee;
    challenge.pool_lamports = 0;
    challenge.completers = 0;
    challenge.deadline_ts = deadline;
    challenge.is_active = true;
    challenge.nonce = nonce;
    challenge.bump = ctx.bumps.challenge;

    Ok(())
}
