pub const ENTRY_FEE_REQUIREMENT_SIZE: usize = 1 + // bump
    32 + // entry_fee_mint
    8 + // min_class_1_fee
    8 + // max_class_1_fee
    8 + // min_class_2_fee
    8 + // max_class_2_fee
    8 + // min_class_3_fee
    8 + // max_class_3_fee
    8 + // min_class_4_fee
    8 + // max_class_4_fee
    8 + // min_class_5_fee
    8; // max_class_5_fee

pub fn lobby_account_size(max_players: u8) -> usize {
    let final_max_players = if max_players < 2 { 2 } else { max_players };
    return 1 + // bump
    1 + // max_players 
    1 + // race_started
    8 + // unlock_time
    1 + // total_laps
    1 + // min_class
    1 + // track_type
    8 + // entry_fee 
    (4 + 32) + // name
    (4 + 32) + // location
    32 + // track_mint
    32 + // entry_fee_mint
    32 + // track_metadata
    32 + // lobby_entry_fee_token
    32 + // lobby_track_token
    32 + // lobby_wsol_token
    32 + // track_holder
    32 + // track_holder_token
    32 + // track_holder_entry_fee_token
    (4 + (32 * final_max_players as usize)); // racers
}

pub const DOGE_RACER_SIZE: usize = 1 + // bump
    32 + // doge_o_pda
    32 + // current_lobby_race
    8 + // last_joined_timestamp
    32 + // doge_holder
    32 + // doge_holder_entry_fee_token
    8 + // total_wins
    8; // total_losses

pub const RACE_STATE_SIZE: usize = 1 + // bump
    32 + // lobby_account
    32 + // entry_fee_mint
    32 + // track_holder
    32 + // track_mint
    32 + // winner_doge_racer_account
    8 + // race_started_at
    8 + // dtrk_fee
    (4 + (32 * 5)); // racers

pub const SOL_NETWORK_FEE: u64 = 10_000_000; // 0.01 SOL network fee 
pub const TRACK_OWNER_PCT: u64 = 20; // 20% track owner share from reward pool

pub const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";
pub const DTRK_MINT: &str = "DTRK1XRNaL6CxfFWwVLZMxyhiCZnwdP32CgzVDXWy5Td";
pub const TRACK_VERIFIED_CREATOR: &str = "E4cAAdwFoJJT7uER46U7CZcrKvYhX8G7BEzC19vuFTMa";
pub const TREASURY_ADDRESS: &str = "GFs7gjW38XBsYbf1G6jkdbiXXkmJ5sKGyakdRmfcFvzN";

// This is the cooldown timer before you can flush a racer from a lobby 
// pub const STALE_RACERS_FLUSH_COOLDOWN: u64 = 20; // 20 seconds 
pub const STALE_RACERS_FLUSH_COOLDOWN: u64 = 60 * 30; // 30 minutes

// This is the cooldown timer before you can close a lobby 
// pub const COOLDOWN_PERIOD: i64 = 10; // 10 seconds
pub const COOLDOWN_PERIOD: i64 = 60 * 60 * 24; // 24 hours
