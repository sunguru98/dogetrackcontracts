use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use doge_o::{get_authority, DogeStats};

use crate::{
    constants::{RACE_STATE_SIZE},
    error::GameError,
    metadata::Metadata,
    state::{DogeRacerState, LobbyState, RaceState},
    utils::*,
};

#[derive(Accounts)]
#[instruction(race_started: u64)]
pub struct CacheRace<'info> {
    #[account(mut, address = get_authority())]
    pub authority: Signer<'info>,

    #[account(
        init,
        space = 8 + RACE_STATE_SIZE,
        payer = authority,
        seeds = [
            b"racedata",
            race_started.to_string().as_bytes(),
            lobby_account.key().as_ref(),
            winner_doge_racer_account.key().as_ref(),
        ],
        bump,
    )]
    pub race_data_state: Account<'info, RaceState>,

    #[account(
        constraint = lobby_account.race_started @ GameError::RaceNotStarted,
        constraint = !is_lobby_empty(&lobby_account.racers) @ GameError::LobbyNotFull,
        seeds = [
            b"lobby",
            track_holder.key().as_ref(),
            track_mint.key().as_ref(),
        ],
        bump = lobby_account.bump,
    )]
    pub lobby_account: Box<Account<'info, LobbyState>>,

    #[account(
        has_one = doge_o_pda,
        constraint = winner_doge_racer_account.last_joined_timestamp.gt(&0) @ GameError::UnauthorizedRacer,
        constraint = winner_doge_racer_account.current_lobby_race.eq(&lobby_account.key()) @ GameError::UnauthorizedRacer,
        seeds = [
            b"dogeracer",
            doge_mint.key().as_ref(),
            doge_o_pda.key().as_ref(),
        ],
        bump = winner_doge_racer_account.bump
    )]
    pub winner_doge_racer_account: Box<Account<'info, DogeRacerState>>,

    #[account(
        has_one = doge_mint,
        has_one = doge_metadata,
        has_one = init_authority,
    )]
    pub doge_o_pda: Box<Account<'info, DogeStats>>,
    pub doge_mint: Account<'info, Mint>,
    pub doge_metadata: Account<'info, Metadata>,

    #[account(
        constraint = track_mint.key().eq(&lobby_account.track_keys.track_mint) @ GameError::UnauthorizedTrackMint
    )]
    pub track_mint: Box<Account<'info, Mint>>,

    #[account(address = lobby_account.track_keys.track_holder)]
    pub track_holder: SystemAccount<'info>,
    pub init_authority: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CacheRace>, race_started: u64) -> Result<()> {
    //require!(!MAINTENANCE_MODE, GameError::GameInMaintenance);

    let racer_state_account = &mut ctx.accounts.race_data_state;
    let winner_doge_racer_account = &ctx.accounts.winner_doge_racer_account;
    let lobby_account = &ctx.accounts.lobby_account;

    racer_state_account.bump = *ctx.bumps.get("race_data_state").unwrap();
    racer_state_account.entry_fee_mint = lobby_account.track_keys.entry_fee_mint.key();
    racer_state_account.lobby_account = lobby_account.key();
    racer_state_account.track_mint = ctx.accounts.track_mint.key();
    racer_state_account.track_holder = ctx.accounts.track_holder.key();
    racer_state_account.doge_racers = lobby_account.racers.clone();
    racer_state_account.entry_fee_token_fee = lobby_account.lobby_data.entry_fee;
    racer_state_account.race_started_at = race_started;
    racer_state_account.winner_doge_racer_account = winner_doge_racer_account.key();

    Ok(())
}
