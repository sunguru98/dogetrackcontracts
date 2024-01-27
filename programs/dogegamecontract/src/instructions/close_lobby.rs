use anchor_lang::prelude::*;
use anchor_spl::token::{
    close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer,
};

use crate::error::GameError;
use crate::utils::*;

use crate::state::LobbyState;

#[derive(Accounts)]
pub struct CloseLobby<'info> {
    #[account(mut)]
    pub track_holder: Signer<'info>,

    #[account(
        address = get_treasury_address(),
    )]
    pub treasury_account: SystemAccount<'info>,

    #[account(
        mut,
        close = track_holder,
        constraint = !lobby_account.race_started @ GameError::RaceAlreadyStarted,
        constraint = is_lobby_empty(&lobby_account.racers) @ GameError::LobbyOccupied,
        constraint = lobby_account.track_keys.track_holder.eq(&track_holder.key()),
        seeds = [
            b"lobby", 
            track_holder.key().as_ref(),
            track_mint.key().as_ref(),
        ],
        bump = lobby_account.bump
    )]
    pub lobby_account: Account<'info, LobbyState>,

    #[account(address = lobby_account.track_keys.track_mint)]
    pub track_mint: Box<Account<'info, Mint>>,

    #[account(address = lobby_account.track_keys.entry_fee_mint)]
    pub entry_fee_mint: Box<Account<'info, Mint>>,

    #[account(address = get_wsol_mint())]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_entry_fee_token,
        constraint = lobby_entry_fee_token.mint.eq(&entry_fee_mint.key()),
        constraint = lobby_entry_fee_token.owner.eq(&lobby_account.key()) @ GameError::InvalidLobbyTokenAccount,
        constraint = lobby_entry_fee_token.amount == 0 @ GameError::LobbyVaultNotEmpty
    )]
    pub lobby_entry_fee_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_wsol_token,
        constraint = lobby_wsol_token.mint.eq(&wsol_mint.key()),
        constraint = lobby_wsol_token.owner.eq(&lobby_account.key()) @ GameError::InvalidLobbyTokenAccount,
    )]
    pub lobby_wsol_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = lobby_wsol_token.mint.eq(&wsol_mint.key()),
        constraint = treasury_wsol_token.owner.eq(&treasury_account.key())
    )]
    pub treasury_wsol_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.track_holder_token,
        constraint = track_holder_token.mint.eq(&track_mint.key()),
        constraint = track_holder_token.owner.eq(&track_holder.key()),
        constraint = track_holder_token.amount == 0
    )]
    pub track_holder_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = lobby_account.track_keys.lobby_track_token,
        constraint = lobby_track_token.mint.eq(&track_mint.key()),
        constraint = lobby_track_token.owner.eq(&lobby_account.key()) @ GameError::InvalidLobbyTokenAccount,
        constraint = lobby_track_token.amount == 1,
    )]
    pub lobby_track_token: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseLobby<'info> {
    pub fn close_token_ctx<'a, 'b, 'c>(
        &self,
        token_account: &Account<'info, TokenAccount>,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, CloseAccount<'info>> {
        CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: token_account.to_account_info(),
                authority: self.lobby_account.to_account_info(),
                destination: self.track_holder.to_account_info(),
            },
            signer_seeds,
        )
    }

    pub fn transfer_token_ctx<'a, 'b, 'c>(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.lobby_track_token.to_account_info(),
                to: self.track_holder_token.to_account_info(),
                authority: self.lobby_account.to_account_info(),
            },
            signer_seeds,
        )
    }

    pub fn transfer_residual_sol_ctx<'a, 'b, 'c>(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.lobby_wsol_token.to_account_info(),
                to: self.treasury_wsol_token.to_account_info(),
                authority: self.lobby_account.to_account_info(),
            },
            signer_seeds,
        )
    }
}

pub fn handler(ctx: Context<CloseLobby>) -> Result<()> {
    let lobby_state_account = &mut ctx.accounts.lobby_account;

    let current_timestamp = (Clock::get()?.unix_timestamp) as u64;
    require!(
        current_timestamp >= lobby_state_account.unlock_time,
        GameError::LobbyLocked
    );

    let track_holder = ctx.accounts.track_holder.key();
    let track_mint = ctx.accounts.track_mint.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"lobby",
        track_holder.as_ref(),
        track_mint.as_ref(),
        &[ctx.accounts.lobby_account.bump],
    ]];

    // Transfer back to track holder
    transfer(ctx.accounts.transfer_token_ctx(signer_seeds), 1)?;

    let lobby_sol_balance = ctx.accounts.lobby_wsol_token.amount;

    if lobby_sol_balance != 0 {
        // Transfer residual SOL to treasury
        transfer(
            ctx.accounts.transfer_residual_sol_ctx(signer_seeds),
            lobby_sol_balance,
        )?;
    }

    msg!("Closing Lobby entry fee token vault");
    // Close Lobby Entry Fee Token
    close_account(
        ctx.accounts
            .close_token_ctx(&ctx.accounts.lobby_entry_fee_token, signer_seeds),
    )?;

    if ctx.accounts.lobby_entry_fee_token.key() != ctx.accounts.lobby_wsol_token.key() {
        msg!("Closing Lobby WSOL Token vault");
        // Close Lobby WSOL Token
        close_account(
            ctx.accounts
                .close_token_ctx(&ctx.accounts.lobby_wsol_token, signer_seeds),
        )?;
    }

    msg!("Closing Lobby Track vault");
    // Close Lobby Track
    close_account(
        ctx.accounts
            .close_token_ctx(&ctx.accounts.lobby_track_token, signer_seeds),
    )?;

    msg!(
        "Closing Lobby account {}",
        ctx.accounts.lobby_account.key().to_string()
    );

    Ok(())
}
