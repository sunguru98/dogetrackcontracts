use anchor_lang::prelude::*;

#[error_code]
pub enum DogeError {
    #[msg("Doge stats should be between 1 to 100")]
    InvalidStats,

    #[msg("Doge Win percentage should be between 1 to 100")]
    InvalidWinPercentage,

    #[msg("Invalid Doge stats for upgrade")]
    InvalidUpdateStats,

    #[msg("Insufficient DTRK for upgrade")]
    InsufficientTokens,

    #[msg("Empty doge token account")]
    EmptyDogeToken,

    #[msg("Invalid doge metadata supplied")]
    InvalidDogeMetadata,

    #[msg("DogeO in maintenance")]
    MaintenanceMode,
}
