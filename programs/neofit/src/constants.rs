use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

pub const SEED_USER_PROFILE: &[u8] = b"user_profile";
pub const SEED_CHALLENGE: &[u8] = b"challenge";
pub const SEED_ENROLLMENT: &[u8] = b"enrollment";

pub const MAX_USERNAME_LEN: usize = 22;
pub const MAX_EXERCISE_ID: u8 = 5;
pub const MAX_REQUIREMENTS: usize = 15;
pub const MAX_EXERCISES_TRACKED: usize = 100;

pub const PROTOCOL_FEE_BPS: u64 = 1000;  // 10%
