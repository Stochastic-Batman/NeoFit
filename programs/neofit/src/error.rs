use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    NotAuthorized,

    #[msg("The challenge has already expired or the deadline has passed.")]
    ChallengeExpired,

    #[msg("Reward has already been claimed for this enrollment.")]
    AlreadyClaimed,

    #[msg("This challenge is currently inactive.")]
    ChallengeInactive,

    #[msg("Insufficient funds to join the challenge.")]
    InsufficientFunds,

    #[msg("The provided exercise ID is invalid.")]
    InvalidExerciseId,

    #[msg("The number of requirements exceeds the maximum allowed.")]
    TooManyRequirements,

    #[msg("The username provided is too long.")]
    UsernameTooLong,

    #[msg("The username provided is too short.")]
    UsernameTooShort,

    #[msg("A mathematical overflow occurred.")]
    Overflow,
}
