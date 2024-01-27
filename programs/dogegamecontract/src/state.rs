use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct LobbyState {
    pub bump: u8,
    pub max_players: u8,
    pub race_started: bool,
    pub unlock_time: u64,
    pub track_keys: TrackKeys,
    pub racers: Vec<Pubkey>,
    pub lobby_data: LobbyData,
}

#[account]
#[derive(Default)]
pub struct RaceState {
    pub bump: u8,
    pub lobby_account: Pubkey,
    pub entry_fee_mint: Pubkey,
    pub track_mint: Pubkey,
    pub track_holder: Pubkey,
    pub winner_doge_racer_account: Pubkey,
    pub race_started_at: u64,
    pub entry_fee_token_fee: u64,
    pub doge_racers: Vec<Pubkey>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct LobbyData {
    pub total_laps: u8,
    pub min_class: u8,
    pub entry_fee: u64,
    pub name: String,
    pub location: String,
    pub track_type: TrackType,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct TrackKeys {
    pub track_mint: Pubkey,
    pub entry_fee_mint: Pubkey,
    pub track_metadata: Pubkey,
    pub lobby_entry_fee_token: Pubkey,
    pub lobby_track_token: Pubkey,
    pub lobby_wsol_token: Pubkey,
    pub track_holder: Pubkey,
    pub track_holder_token: Pubkey,
    pub track_holder_entry_fee_token: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TrackType {
    Dirt,
    Space,
    Pavement,
    Sand,
}

impl Default for TrackType {
    fn default() -> Self {
        TrackType::Sand
    }
}

#[account]
#[derive(Default)]
pub struct DogeRacerState {
    // Doge State Bump
    pub bump: u8,
    pub doge_o_pda: Pubkey,
    pub current_lobby_race: Pubkey,
    pub last_joined_timestamp: u64,
    pub doge_holder_entry_fee_token: Pubkey,
    pub total_wins: u64,
    pub total_losses: u64,
}

#[account]
#[derive(Default)]
pub struct EntryFeeRequirementsState {
    pub bump: u8,
    pub entry_fee_mint: Pubkey,
    pub entry_fee_requirements: EntryFeeRequirements,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct EntryFeeRequirements {
    pub min_fee: u64,
    pub max_class_1_fee: u64,
    pub max_class_2_fee: u64,
    pub max_class_3_fee: u64,
    pub max_class_4_fee: u64,
    pub max_class_5_fee: u64,
}
