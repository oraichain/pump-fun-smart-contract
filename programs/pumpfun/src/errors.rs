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

    #[msg("IncorrectAuthority")]
    IncorrectAuthority,

    #[msg("Overflow or underflow occured")]
    OverflowOrUnderflowOccurred,

    #[msg("Amount is invalid")]
    InvalidAmount,

    #[msg("Incorrect team wallet address")]
    IncorrectTeamWallet,
    
    #[msg("Curve is not completed")]
    CurveNotCompleted,
    
    #[msg("Mint authority should be revoked")]
    MintAuthorityEnabled,
    
    #[msg("Freeze authority should be revoked")]
    FreezeAuthorityEnabled,
}
