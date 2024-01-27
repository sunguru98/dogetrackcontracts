use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer as TokenTransfer};
use doge_o::DogeStats;

use crate::{
    constants::{SOL_NETWORK_FEE},
    error::GameError,
    metadata::Metadata,
    state::{DogeRacerState, LobbyState},
    utils::*,
};

#[derive(Accounts)]
pub struct JoinRace<'info> {
    #[account(mut)]
    pub doge_holder: Signer<'info>,
    #[account(address = doge_o_pda.init_authority)]
    pub init_authority: SystemAccount<'info>,

    #[account(
        constraint = track_holder.key().eq(&lobby_account.track_keys.track_holder) @ GameError::UnauthorizedTrackHolder
    )]
    pub track_holder: SystemAccount<'info>,

    #[account(
        mut,
        constraint = doge_racer_account.last_joined_timestamp.eq(&0) @ GameError::DogeInRace,
        constraint = doge_racer_account.current_lobby_race.eq(&Pubkey::default()) @ GameError::DogeInRace,
        has_one = doge_o_pda,
        seeds = [
            b"dogeracer",
            doge_mint.key().as_ref(),
            doge_o_pda.key().as_ref(),
        ],
        bump = doge_racer_account.bump
    )]
    pub doge_racer_account: Box<Account<'info, DogeRacerState>>,

    #[account(
        mut,
        seeds = [
            b"lobby", 
            track_holder.key().as_ref(),
            track_mint.key().as_ref(),
        ],
        bump = lobby_account.bump,
        constraint = lobby_account.race_started.eq(&false) @ GameError::RaceAlreadyStarted,
        constraint = is_lobby_empty(&lobby_account.racers).eq(&true) @ GameError::LobbyFull
    )]
    pub lobby_account: Box<Account<'info, LobbyState>>,

    #[account(
        has_one = doge_mint,
        has_one = doge_metadata,
        has_one = init_authority,
    )]
    pub doge_o_pda: Box<Account<'info, DogeStats>>,

    #[account(
        constraint = doge_token_account.owner.eq(&doge_holder.key()) @ GameError::UnauthorizedRacer,
        constraint = doge_token_account.amount.eq(&1) @ GameError::UnauthorizedRacer,
        constraint = doge_token_account.mint.eq(&doge_mint.key()) @ GameError::UnauthorizedRacer
    )]
    pub doge_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = track_mint.key().eq(&lobby_account.track_keys.track_mint) @ GameError::UnauthorizedTrackMint
    )]
    pub track_mint: Box<Account<'info, Mint>>,
    pub doge_mint: Box<Account<'info, Mint>>,
    pub doge_metadata: Box<Account<'info, Metadata>>,

    // WSOL related
    #[account(address = get_wsol_mint())]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = doge_holder_wsol.owner.eq(&doge_holder.key()),
        constraint = doge_holder_wsol.mint.eq(&wsol_mint.key()),
        constraint = doge_holder_wsol.amount.ge(&SOL_NETWORK_FEE) @ GameError::InsufficientSOL,
    )]
    pub doge_holder_wsol: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_wsol_token @ GameError::InvalidLobbyTokenAccount,
    )]
    pub lobby_wsol_token: Account<'info, TokenAccount>,

    // Entry Fee mint related
    #[account(address = lobby_account.track_keys.entry_fee_mint)]
    pub entry_fee_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = doge_holder_entry_fee_token.owner.eq(&doge_holder.key()),
        constraint = doge_holder_entry_fee_token.mint.eq(&entry_fee_mint.key()),
        constraint = doge_holder_entry_fee_token.amount.ge(&lobby_account.lobby_data.entry_fee) @ GameError::InsufficientEntryFeeTokenBalance,
    )]
    pub doge_holder_entry_fee_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_entry_fee_token @ GameError::InvalidLobbyTokenAccount,
    )]
    pub lobby_entry_fee_token: Account<'info, TokenAccount>,

    // Programs
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<JoinRace>) -> Result<()> {
    //require!(!MAINTENANCE_MODE, GameError::GameInMaintenance);

    let lobby_account = &mut ctx.accounts.lobby_account;
    let doge_racer_account = &mut ctx.accounts.doge_racer_account;

    let doge_holder = &ctx.accounts.doge_holder;
    let doge_holder_entry_fee_token = &ctx.accounts.doge_holder_entry_fee_token;
    let doge_holder_wsol = &ctx.accounts.doge_holder_wsol;
    let lobby_entry_fee_token = &ctx.accounts.lobby_entry_fee_token;
    let lobby_wsol_token = &ctx.accounts.lobby_wsol_token;

    require!(
        is_doge_stats_valid(
            &ctx.accounts.init_authority.key(),
            &ctx.accounts.doge_mint.key(),
            &ctx.accounts.doge_o_pda.key()
        ),
        GameError::InvalidDogeStats
    );

    if let Some(racer_index) = find_racer_index(&lobby_account.racers, None) {
        // State changes
        // 1. Adding to lobby account racers
        msg!(
            "Joining Race {} as position no: {}",
            lobby_account.key().to_string(),
            racer_index + 1
        );
        lobby_account.racers[racer_index] = doge_racer_account.key();

        // 2. Updating timestamp to doge racer
        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        msg!("Joining Racer at {}", current_timestamp);
        doge_racer_account.current_lobby_race = lobby_account.key();
        doge_racer_account.last_joined_timestamp = current_timestamp;
        doge_racer_account.doge_holder_entry_fee_token = doge_holder_entry_fee_token.key();

        // Transferring WSOL
        msg!("Collecting Entry fee token and WSOL fees");
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    authority: doge_holder.to_account_info(),
                    from: doge_holder_wsol.to_account_info(),
                    to: lobby_wsol_token.to_account_info(),
                },
            ),
            SOL_NETWORK_FEE,
        )?;

        // Transferring Entry Fee Mint
        let entry_fee_mint_joining_fee = lobby_account.lobby_data.entry_fee;

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    authority: doge_holder.to_account_info(),
                    from: doge_holder_entry_fee_token.to_account_info(),
                    to: lobby_entry_fee_token.to_account_info(),
                },
            ),
            entry_fee_mint_joining_fee,
        )?;

        // Checking if lobby is full and flipping the toggle accordingly
        lobby_account.race_started = !is_lobby_empty(&lobby_account.racers);
        msg!("Race Started: {}", lobby_account.race_started);

        Ok(())
    } else {
        return err!(GameError::LobbyOccupied);
    }
}
