use anchor_lang::prelude::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExerciseRequirement {
    pub exercise_id: u8,
    pub rep_target: u16,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ExerciseCount {
    pub exercise_id: u8,
    pub count: u32,
}


#[account]
pub struct UserProfile {  // 8 + 32 + 26 + 8 + 504 + 8 + 4 + 1 = 591 bytes
    pub authority: Pubkey,
    pub username: String,
    pub total_reps: u64,
    pub rep_counts: Vec<ExerciseCount>,
    pub last_workout_ts: i64,
    pub streak_days: u32,
    pub bump: u8,
}


#[account]
pub struct Challenge {  // 8 + 32 + 36 + 49 + 8 + 8 + 4 + 8 + 1 + 8 + 1 = 163 bytes
    pub authority: Pubkey,
    pub title: String,
    pub requirements: Vec<ExerciseRequirement>,
    pub entry_fee_lamports: u64,
    pub pool_lamports: u64,
    pub completers: u32,
    pub deadline_ts: i64,
    pub is_active: bool,
    pub nonce: u64,
    pub bump: u8,
}


#[account]
pub struct Enrollment {  // 8 + 32 + 32 + 79 + 1 + 1 + 1 = 154 bytes
    pub user: Pubkey,
    pub challenge: Pubkey,
    pub reps_logged: Vec<ExerciseCount>,
    pub completed: bool,
    pub reward_claimed: bool,
    pub bump: u8,
}
