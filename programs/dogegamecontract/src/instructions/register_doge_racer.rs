use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use doge_o::DogeStats;

use crate::{
    constants::DOGE_RACER_SIZE, error::GameError, metadata::Metadata, state::DogeRacerState,
    utils::is_doge_stats_valid,
};

#[derive(Accounts)]
pub struct RegisterDogeRacer<'info> {
    #[account(mut)]
    pub doge_holder: Signer<'info>,
    // Doge Racer State account
    #[account(
        init,
        space = 8 + DOGE_RACER_SIZE,
        payer = doge_holder,
        seeds = [
            b"dogeracer",
            doge_mint.key().as_ref(),
            doge_o_pda.key().as_ref(),
        ],
        bump
    )]
    pub doge_racer_account: Box<Account<'info, DogeRacerState>>,
    // Doge O related accounts
    pub init_authority: SystemAccount<'info>,
    #[account(
        has_one = doge_mint,
        has_one = doge_metadata,
        has_one = init_authority,
    )]
    pub doge_o_pda: Box<Account<'info, DogeStats>>,
    #[account(
        constraint = doge_token_account.owner.eq(&doge_holder.key()) @ GameError::UnauthorizedRacer,
        constraint = doge_token_account.mint.eq(&doge_mint.key()) @ GameError::UnauthorizedRacer,
        constraint = doge_token_account.amount.eq(&1) @ GameError::UnauthorizedRacer
    )]
    pub doge_token_account: Account<'info, TokenAccount>,
    pub doge_mint: Account<'info, Mint>,
    pub doge_metadata: Account<'info, Metadata>,
    // programs
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterDogeRacer>) -> Result<()> {
    //require!(!MAINTENANCE_MODE, GameError::GameInMaintenance);
    require!(
        is_doge_stats_valid(
            &ctx.accounts.init_authority.key(),
            &ctx.accounts.doge_mint.key(),
            &ctx.accounts.doge_o_pda.key()
        ),
        GameError::InvalidDogeStats
    );

    let doge_racer_account = &mut ctx.accounts.doge_racer_account;

    doge_racer_account.bump = *ctx.bumps.get("doge_racer_account").unwrap();
    doge_racer_account.current_lobby_race = Pubkey::default();
    doge_racer_account.doge_holder_entry_fee_token = Pubkey::default();
    doge_racer_account.last_joined_timestamp = 0u64;
    doge_racer_account.doge_o_pda = ctx.accounts.doge_o_pda.key();

    Ok(())
}
