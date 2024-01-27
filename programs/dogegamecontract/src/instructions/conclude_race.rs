use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use doge_o::{
    cpi::{accounts::SetWinPercentage, set_win_percentage},
    get_authority,
    program::DogeO,
    DogeStats,
};

use crate::{
    constants::{SOL_NETWORK_FEE, TRACK_OWNER_PCT},
    error::GameError,
    metadata::Metadata,
    state::*,
    utils::*,
};

#[derive(Accounts)]
pub struct ConcludeRace<'info> {
    #[account(
        mut,
        address = get_authority()
    )]
    pub authority: Signer<'info>,
    pub init_authority: SystemAccount<'info>,

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
        mut,
        has_one = doge_mint,
        has_one = doge_metadata,
        has_one = init_authority,
    )]
    pub doge_o_pda: Box<Account<'info, DogeStats>>,
    pub doge_mint: Box<Account<'info, Mint>>,
    pub doge_metadata: Account<'info, Metadata>,
    #[account(
        address = lobby_account.track_keys.entry_fee_mint
    )]
    pub entry_fee_mint: Box<Account<'info, Mint>>,
    #[account(
        address = get_wsol_mint()
    )]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = track_mint.key().eq(&lobby_account.track_keys.track_mint) @ GameError::UnauthorizedTrackMint
    )]
    pub track_mint: Box<Account<'info, Mint>>,

    /// CHECK: Proper PDA checks have been made in the handler function
    #[account(
        mut, 
        address = doge_racer_account.doge_holder_entry_fee_token
    )]
    pub doge_holder_entry_fee_token: UncheckedAccount<'info>,

    /// CHECK: Proper PDA checks have been made in the handler function
    #[account(
        mut,
        address = lobby_account.track_keys.track_holder_entry_fee_token
    )]
    pub track_holder_entry_fee_token: UncheckedAccount<'info>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_entry_fee_token,
        constraint = lobby_entry_fee_token.owner.eq(&lobby_account.key()),
        constraint = lobby_entry_fee_token.mint.eq(&entry_fee_mint.key()),
    )]
    pub lobby_entry_fee_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_wsol_token,
        constraint = lobby_wsol_token.owner.eq(&lobby_account.key()),
        constraint = lobby_wsol_token.mint.eq(&wsol_mint.key()),
    )]
    pub lobby_wsol_token: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub doge_o_program: Program<'info, DogeO>,

    #[account(
        mut,
        constraint = treasury_wsol_token.owner.eq(&treasury_address.key()),
        constraint = treasury_wsol_token.mint.eq(&wsol_mint.key())
    )]
    pub treasury_wsol_token: Account<'info, TokenAccount>,

    #[account(address = get_treasury_address())]
    pub treasury_address: SystemAccount<'info>,
    #[account(address = lobby_account.track_keys.track_holder)]
    pub track_holder: SystemAccount<'info>,
    pub doge_holder: SystemAccount<'info>,
}

pub fn handler(ctx: Context<ConcludeRace>, is_winner: bool, new_win_pct: u8) -> Result<()> {
    //require!(!MAINTENANCE_MODE, GameError::GameInMaintenance);

    // All accounts
    let treasury_wsol_token = &mut ctx.accounts.treasury_wsol_token;

    let lobby_account = &mut ctx.accounts.lobby_account;
    let doge_racer_account = &mut ctx.accounts.doge_racer_account;
    let track_mint = ctx.accounts.track_mint.key();
    let entry_fee_mint = &ctx.accounts.entry_fee_mint;

    let lobby_entry_fee_token = &mut ctx.accounts.lobby_entry_fee_token;
    let lobby_wsol_token = &mut ctx.accounts.lobby_wsol_token;

    let track_holder_entry_fee_token = &ctx.accounts.track_holder_entry_fee_token;
    let doge_holder_entry_fee_token = &ctx.accounts.doge_holder_entry_fee_token;

    let track_holder = ctx.accounts.track_holder.key();
    let doge_holder = ctx.accounts.doge_holder.key();

    let token_program = &ctx.accounts.token_program;
    let lobby_entry_fee_token_balance = lobby_entry_fee_token.amount;
    let lobby_wsol_token_balance = lobby_wsol_token.amount;

    // Validations
    let is_valid_racer = lobby_account
        .racers
        .iter()
        .any(|&racer| racer.eq(&doge_racer_account.key()));

    require!(is_valid_racer, GameError::UnauthorizedRacer);

    require!(
        is_doge_stats_valid(
            &ctx.accounts.init_authority.key(),
            &ctx.accounts.doge_mint.key(),
            &ctx.accounts.doge_o_pda.key()
        ),
        GameError::InvalidDogeStats
    );

    if is_winner {
        // Entry fee token validation checks
        check_valid_ata(
            &track_holder_entry_fee_token,
            &track_holder,
            &entry_fee_mint.key(),
        )?;

        check_valid_ata(
            &doge_holder_entry_fee_token,
            &doge_holder,
            &entry_fee_mint.key(),
        )?;

        msg!(
            "Winner is {} doge racer {}. Doge Holder {}",
            &ctx.accounts.doge_metadata.data.name,
            doge_racer_account.key().to_string(),
            &ctx.accounts.doge_holder.key().to_string()
        );

        let total_entry_fee = lobby_account
            .lobby_data
            .entry_fee
            .checked_mul(lobby_account.max_players.into())
            .ok_or(GameError::MathOverflow)?;

        let total_network_fee = SOL_NETWORK_FEE
            .checked_mul(lobby_account.max_players.into())
            .ok_or(GameError::MathOverflow)?;

        msg!(
            "Expected total entry fee collected {}\n, Lobby entry fee token balance {}\n, Lobby WSOL token balance {}",
            total_entry_fee,
            lobby_entry_fee_token_balance,
            lobby_wsol_token_balance,
        );

        require!(
            lobby_entry_fee_token_balance >= total_entry_fee,
            GameError::InsufficientEntryFeeTokenBalance
        );

        require!(
            lobby_wsol_token_balance >= total_network_fee,
            GameError::InsufficientSOL
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"lobby",
            track_holder.as_ref(),
            track_mint.as_ref(),
            &[lobby_account.bump],
        ]];

        // 1. Transferring Entry Fee Token to track owner (20%)
        let pool_share = if entry_fee_mint.key().eq(&get_wsol_mint()) {
            lobby_entry_fee_token_balance - total_network_fee
        } else {
            lobby_entry_fee_token_balance
        };

        let track_owner_transfer_amount = percentage_of(pool_share, TRACK_OWNER_PCT)?;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    authority: lobby_account.to_account_info(),
                    from: lobby_entry_fee_token.to_account_info(),
                    to: track_holder_entry_fee_token.to_account_info(),
                },
                signer_seeds,
            ),
            track_owner_transfer_amount,
        )?;

        let winner_transfer_amount = pool_share.saturating_sub(track_owner_transfer_amount);

        msg!(
            "Transferring {} Entry Fee Token to track owner and {} Entry Fee Token to winner",
            track_owner_transfer_amount,
            winner_transfer_amount
        );

        // 2. Transferring entry fee mint to winner (80%)
        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    authority: lobby_account.to_account_info(),
                    from: lobby_entry_fee_token.to_account_info(),
                    to: doge_holder_entry_fee_token.to_account_info(),
                },
                signer_seeds,
            ),
            winner_transfer_amount,
        )?;

        // 3. Transfer race WSOL fees to treasury
        msg!("Transferring network fees to treasury");
        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    authority: lobby_account.to_account_info(),
                    from: lobby_wsol_token.to_account_info(),
                    to: treasury_wsol_token.to_account_info(),
                },
                signer_seeds,
            ),
            total_network_fee,
        )?;

        doge_racer_account.total_wins += 1;
    } else {
        doge_racer_account.total_losses += 1;
    }

    if let Some(racer_index) =
        find_racer_index(&lobby_account.racers, Some(doge_racer_account.key()))
    {
        msg!("Racer Index {}", racer_index);

        if racer_index == (lobby_account.max_players - 1).into() {
            msg!("Resetting lobby {}!", lobby_account.key().to_string());
            lobby_account.race_started = false;
            lobby_account.racers = fill_empty_racers(lobby_account.max_players);
        }
    }

    // 4. Clearing out racer's and lobby's data
    msg!("Resetting racer {}!", doge_racer_account.key().to_string());
    doge_racer_account.last_joined_timestamp = 0;
    doge_racer_account.current_lobby_race = Pubkey::default();
    doge_racer_account.doge_holder_entry_fee_token = Pubkey::default();

    // 5. Updating corresponding doge o stats
    set_win_percentage(
        CpiContext::new(
            ctx.accounts.doge_o_program.to_account_info(),
            SetWinPercentage {
                doge_mint: ctx.accounts.doge_mint.to_account_info(),
                doge_stats: ctx.accounts.doge_o_pda.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                init_authority: ctx.accounts.init_authority.to_account_info(),
            },
        ),
        new_win_pct,
    )?;

    Ok(())
}
