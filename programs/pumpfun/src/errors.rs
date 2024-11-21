use anchor_lang::prelude::*;

pub use PumpfunError::*;

#[error_code]
pub enum PumpfunError {
    #[msg("ValueTooSmall")]
    ValueTooSmall,

    #[msg("ValueTooLarge")]
    ValueTooLarge,

    #[msg("ValueInvalid")]
    ValueInvalid,

    #[msg("SerializationFailed")]
    SerializationFailed,

    #[msg("IncorrectAuthority")]
    IncorrectAuthority,

    #[msg("NothingToDo")]
    NothingToDo,
    
    #[msg("Program is in an invalid state")]
    InvalidState,

    #[msg("Overflow or underflow occured")]
    OverflowOrUnderflowOccurred,

    #[msg("Amount is invalid")]
    InvalidAmount,

    #[msg("Incorrect team wallet address")]
    IncorrectTeamWallet,
    
    #[msg("Curve is not completed")]
    CurveNotCompleted,
}
