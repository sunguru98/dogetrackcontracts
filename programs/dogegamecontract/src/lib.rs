use anchor_lang::prelude::*;

use instructions::*;
use state::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod metadata;
pub mod state;
pub mod utils;

declare_id!("GAmedvouiMuUop6UUvGe8L5wdAQaSwXa77JPxs87pYpE");

#[program]
pub mod dogegamecontract {
    use super::*;

    pub fn create_lobby(
        ctx: Context<CreateLobby>,
        lobby_metadata: LobbyData,
        max_players: u8,
    ) -> Result<()> {
        instructions::create_lobby::handler(ctx, lobby_metadata, max_players)
    }

    pub fn close_lobby(ctx: Context<CloseLobby>) -> Result<()> {
        instructions::close_lobby::handler(ctx)
    }

    pub fn register_doge_racer(ctx: Context<RegisterDogeRacer>) -> Result<()> {
        instructions::register_doge_racer::handler(ctx)
    }

    pub fn join_race(ctx: Context<JoinRace>) -> Result<()> {
        instructions::join_race::handler(ctx)
    }

    pub fn leave_race(ctx: Context<LeaveRace>) -> Result<()> {
        instructions::leave_race::handler(ctx)
    }

    pub fn flush_stale_racer(ctx: Context<FlushStaleRacer>) -> Result<()> {
        instructions::flush_stale_racer::handler(ctx)
    }

    pub fn conclude_race(
        ctx: Context<ConcludeRace>,
        is_winner: bool,
        new_win_pct: u8,
    ) -> Result<()> {
        instructions::conclude_race::handler(ctx, is_winner, new_win_pct)
    }

    pub fn cache_race(ctx: Context<CacheRace>, race_started: u64) -> Result<()> {
        instructions::cache_race::handler(ctx, race_started)
    }

    pub fn init_entry_fee_requirements(
        ctx: Context<InitEntryFeeRequirements>,
        entry_fee_requirements: EntryFeeRequirements,
    ) -> Result<()> {
        instructions::init_entry_fee_requirements::handler(ctx, entry_fee_requirements)
    }

    pub fn update_entry_fee_requirements(
        ctx: Context<UpdateEntryFeeRequirements>,
        new_entry_fee_requirements: EntryFeeRequirements,
    ) -> Result<()> {
        instructions::update_entry_fee_requirements::handler(ctx, new_entry_fee_requirements)
    }

    /* Maintenance mode instructions */
    pub fn admin_close_lobby(ctx: Context<AdminCloseLobby>) -> Result<()> {
        instructions::admin_close_lobby::handler(ctx)
    }

    pub fn admin_close_doge_racer(ctx: Context<CloseDogeRacer>) -> Result<()> {
        instructions::admin_close_doge_racer::handler(ctx)
    }

    pub fn admin_close_race_state(
        ctx: Context<AdminCloseRaceState>,
        race_started: u64,
    ) -> Result<()> {
        instructions::admin_close_race_state::handler(ctx, race_started)
    }

    pub fn admin_close_entry_fee_requirements(
        ctx: Context<AdminCloseEntryFeeRequirements>,
    ) -> Result<()> {
        instructions::admin_close_entry_fee_requirments::handler(ctx)
    }
}
