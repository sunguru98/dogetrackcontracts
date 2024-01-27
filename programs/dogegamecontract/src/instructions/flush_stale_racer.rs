use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer as TokenTransfer};
use doge_o::{get_authority, DogeStats};

use crate::{
    constants::{SOL_NETWORK_FEE, STALE_RACERS_FLUSH_COOLDOWN},
    error::GameError,
    metadata::Metadata,
    state::{DogeRacerState, LobbyState},
    utils::*,
};

#[derive(Accounts)]
pub struct FlushStaleRacer<'info> {
    pub doge_holder: SystemAccount<'info>,

    #[account(mut, address = get_authority())]
    pub authority: Signer<'info>,

    #[account(address = doge_o_pda.init_authority)]
    pub init_authority: SystemAccount<'info>,

    #[account(
        constraint = track_holder.key().eq(&lobby_account.track_keys.track_holder) @ GameError::UnauthorizedTrackHolder
    )]
    pub track_holder: SystemAccount<'info>,

    #[account(
        mut,
        has_one = doge_o_pda,
        constraint = doge_racer_account.last_joined_timestamp.gt(&0) @ GameError::UnauthorizedRacer,
        constraint = doge_racer_account.current_lobby_race.eq(&lobby_account.key()) @ GameError::UnauthorizedRacer,
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
        constraint = lobby_account.race_started.eq(&false) @ GameError::RaceAlreadyStarted
    )]
    pub lobby_account: Box<Account<'info, LobbyState>>,

    #[account(
        has_one = doge_mint,
        has_one = doge_metadata,
        has_one = init_authority,
    )]
    pub doge_o_pda: Box<Account<'info, DogeStats>>,

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
        constraint = doge_holder_wsol.mint.eq(&wsol_mint.key())
    )]
    pub doge_holder_wsol: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_wsol_token @ GameError::InvalidLobbyTokenAccount,
        constraint = lobby_wsol_token.amount.ge(&SOL_NETWORK_FEE) @ GameError::InsufficientSOL,
    )]
    pub lobby_wsol_token: Box<Account<'info, TokenAccount>>,

    // Entry fee related
    #[account(
        mut,
        address = lobby_account.track_keys.entry_fee_mint
    )]
    pub entry_fee_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = doge_holder_entry_fee_token.owner.eq(&doge_holder.key()),
        constraint = doge_holder_entry_fee_token.mint.eq(&entry_fee_mint.key())
    )]
    pub doge_holder_entry_fee_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lobby_entry_fee_token.key().eq(&lobby_account.track_keys.lobby_entry_fee_token) @ GameError::InvalidLobbyTokenAccount,
        constraint = lobby_entry_fee_token.amount.ge(&lobby_account.lobby_data.entry_fee) @ GameError::InsufficientEntryFeeTokenBalance,
    )]
    pub lobby_entry_fee_token: Account<'info, TokenAccount>,

    // Programs
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FlushStaleRacer>) -> Result<()> {
    let lobby_account = &mut ctx.accounts.lobby_account;
    let doge_racer_account = &mut ctx.accounts.doge_racer_account;
    let lobby_entry_fee_token = &ctx.accounts.lobby_entry_fee_token;
    let lobby_wsol_token = &ctx.accounts.lobby_wsol_token;
    let doge_holder_entry_fee_token = &ctx.accounts.doge_holder_entry_fee_token;
    let doge_holder_wsol = &&ctx.accounts.doge_holder_wsol;

    let track_holder = &ctx.accounts.track_holder.key();
    let track_mint = &ctx.accounts.track_mint.key();

    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    let racer_join_time_delta = current_timestamp
        .checked_sub(doge_racer_account.last_joined_timestamp)
        .ok_or(GameError::MathOverflow)?;

    require!(
        racer_join_time_delta.ge(&STALE_RACERS_FLUSH_COOLDOWN),
        GameError::RacerNotStale
    );

    require!(
        is_doge_stats_valid(
            &ctx.accounts.init_authority.key(),
            &ctx.accounts.doge_mint.key(),
            &ctx.accounts.doge_o_pda.key()
        ),
        GameError::InvalidDogeStats
    );

    if let Some(racer_index) =
        find_racer_index(&lobby_account.racers, Some(doge_racer_account.key()))
    {
        // State changes
        // 1. Remove racer from the lobby
        lobby_account.racers[racer_index] = Pubkey::default();
        lobby_account.race_started = false;

        // 2. Nullify doge racer state to default
        doge_racer_account.current_lobby_race = Pubkey::default();
        doge_racer_account.doge_holder_entry_fee_token = Pubkey::default();
        doge_racer_account.last_joined_timestamp = 0;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"lobby",
            track_holder.as_ref(),
            track_mint.as_ref(),
            &[lobby_account.bump],
        ]];

        // Transfers
        // 1. WSOL Transfer
        let amount_to_transfer = SOL_NETWORK_FEE;
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: lobby_wsol_token.to_account_info(),
                    to: doge_holder_wsol.to_account_info(),
                    authority: lobby_account.to_account_info(),
                },
                signer_seeds,
            ),
            amount_to_transfer,
        )?;

        // 2. Entry fee token transfer
        let entry_fee_token_joining_fee = lobby_account.lobby_data.entry_fee;

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: lobby_entry_fee_token.to_account_info(),
                    to: doge_holder_entry_fee_token.to_account_info(),
                    authority: lobby_account.to_account_info(),
                },
                signer_seeds,
            ),
            entry_fee_token_joining_fee,
        )?;

        Ok(())
    } else {
        return err!(GameError::InvalidDogePosition);
    }
}
