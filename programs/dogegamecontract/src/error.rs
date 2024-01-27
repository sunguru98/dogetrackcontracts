use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    // 6000
    #[msg("Math Overflow")]
    MathOverflow,

    // 6001
    #[msg("Invalid Track")]
    InvalidTrack,

    // 6002
    #[msg("Unauthorized Track Holder")]
    UnauthorizedTrackHolder,

    // 6003
    #[msg("Unauthorized Track Mint")]
    UnauthorizedTrackMint,

    // 6004
    #[msg("Invalid Lobby Metadata")]
    InvalidLobbyMetadata,

    // 6005
    #[msg("Invalid Racer Name")]
    InvalidRacerName,

    // 6006
    #[msg("Invalid Doge Stats")]
    InvalidDogeStats,

    // 6007
    #[msg("Unauthorized Doge Racer")]
    UnauthorizedRacer,

    // 6008
    #[msg("Lobby account locked")]
    LobbyLocked,

    // 6009
    #[msg("Lobby contains doges")]
    LobbyOccupied,

    // 6010
    #[msg("Lobby is not full")]
    LobbyNotFull,

    // 6011
    #[msg("Lobby is full")]
    LobbyFull,

    // 6012
    #[msg("Lobby Entry Fee Mint vault not empty")]
    LobbyVaultNotEmpty,

    // 6013
    #[msg("Invalid Lobby Token account")]
    InvalidLobbyTokenAccount,

    // 6014
    #[msg("Insufficient Entry Fee Mint Token Balance")]
    InsufficientEntryFeeTokenBalance,

    // 6015
    #[msg("Insufficient SOL")]
    InsufficientSOL,

    // 6016
    #[msg("Invalid Doge Position")]
    InvalidDogePosition,

    // 6017
    #[msg("Cannot leave a race which has started already")]
    RaceAlreadyStarted,

    // 6018
    #[msg("Cannot conclude winner before race started")]
    RaceNotStarted,

    // 6019
    #[msg("Doge Already in Race. Cannot join twice")]
    DogeInRace,

    // 6020
    #[msg("Racer is not stale to be flushed")]
    RacerNotStale,

    // 6021
    #[msg("Invalid Entry fee requirment")]
    InvalidEntryFeeRequirement,

    // 6022
    #[msg("Only Fungible tokens are allowed as entry fee mint")]
    CannotAcceptNFTAsEntryFee,

    // 6023
    #[msg("Maintenance mode turned on")]
    GameInMaintenance,

    // 6024
    #[msg("Maintenance mode turned off")]
    GameNotInMaintenance,

    // 6025
    #[msg("Invalid Entry fee mint")]
    InvalidEntryFeeMint,

    // 6026
    #[msg("Max players cannot be less than 2")]
    InvalidMaxPlayersCount,
}
