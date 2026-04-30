pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5");

#[program]
pub mod neofit {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        initialize_user::handler(ctx)
    }

    pub fn update_username(ctx: Context<UpdateUsername>, new_username: String) -> Result<()> {
        update_username::handler(ctx, new_username)
    }   

    pub fn log_reps(ctx: Context<LogReps>, exercise_id: u8, count: u32) -> Result<()> {
        log_reps::handler(ctx, exercise_id, count)
    }

    pub fn create_challenge(ctx: Context<CreateChallenge>, title: String, requirements: Vec<ExerciseRequirement>, entry_fee: u64, deadline: i64, nonce; u64) -> Result<()> {
        create_challenge::handler(ctx, title, requirements, entry_fee, deadline, nonce)
    } 
}
