use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct LogReps<'info> {
    #[account(
        mut,
        seeds = [SEED_USER_PROFILE, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_profile: Account<'info, UserProfile>,

    // Optional: Only provided if logging reps for a specific challenge
    #[account(mut)]
    pub enrollment: Option<Account<'info, Enrollment>>,

    #[account(mut)]
    pub challenge: Option<Account<'info, Challenge>>,

    pub authority: Signer<'info>,
}


pub fn handler(ctx: Context<LogReps>, exercise_id: u8, count: u32) -> Result<()> {
    require!(exercise_id <= MAX_EXERCISE_ID, ErrorCode::InvalidExerciseId);

    let user_profile = &mut ctx.accounts.user_profile;
    let mut profile_exercise_exists = false;
    
    user_profile.total_reps = user_profile.total_reps.checked_add(count as u64).ok_or(ErrorCode::Overflow)?;

    for exercise in user_profile.rep_counts.iter_mut() {
        if exercise.exercise_id == exercise_id {
            exercise.count = exercise.count.checked_add(count).ok_or(ErrorCode::Overflow)?;
            profile_exercise_exists = true;
            break;
        }
    }

    if !profile_exercise_exists {
        require!(user_profile.rep_counts.len() < MAX_EXERCISES_TRACKED, ErrorCode::Overflow);
        user_profile.rep_counts.push(ExerciseCount { exercise_id, count });
    }

    if let (Some(enrollment), Some(challenge)) = (&mut ctx.accounts.enrollment, &mut ctx.accounts.challenge) {
        require!(challenge.is_active, ErrorCode::ChallengeInactive);
        require!(Clock::get()?.unix_timestamp <= challenge.deadline_ts, ErrorCode::ChallengeExpired);
        require!(enrollment.user == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        require!(enrollment.challenge == challenge.key(), ErrorCode::NotAuthorized);
        
        if !enrollment.completed {
            let mut enrollment_exercise_exists = false;
            for exercise in enrollment.reps_logged.iter_mut() {
                if exercise.exercise_id == exercise_id {
                    exercise.count = exercise.count.checked_add(count).ok_or(ErrorCode::Overflow)?;
                    enrollment_exercise_exists = true;
                    break;
                }
            }

            if !enrollment_exercise_exists {
                require!(enrollment.reps_logged.len() < MAX_REQUIREMENTS, ErrorCode::TooManyRequirements);
                enrollment.reps_logged.push(ExerciseCount { exercise_id, count });
            }

            let mut all_requirements_met = true;
            for req in challenge.requirements.iter() {
                let logged = enrollment.reps_logged.iter().find(|r| r.exercise_id == req.exercise_id);
                
                match logged {
                    Some(l) if l.count >= req.rep_target as u32 => { continue; },
                    _ => { all_requirements_met = false; break; }
                }
            }

            if all_requirements_met {
                enrollment.completed = true;
                challenge.completers = challenge.completers.checked_add(1).ok_or(ErrorCode::Overflow)?;
            }
        }
    }

    Ok(())
}
