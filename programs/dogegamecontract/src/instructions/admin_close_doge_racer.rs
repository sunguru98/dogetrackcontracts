use anchor_lang::prelude::*;
use doge_o::get_authority;

use crate::state::DogeRacerState;

#[derive(Accounts)]
pub struct CloseDogeRacer<'info> {
    #[account(
        mut,
        address = get_authority()
    )]
    pub authority: Signer<'info>,

    // Doge Racer State account
    #[account(
        mut,
        close = authority,
    )]
    pub doge_racer_account: Box<Account<'info, DogeRacerState>>,
}

pub fn handler(_: Context<CloseDogeRacer>) -> Result<()> {
    //require!(MAINTENANCE_MODE, GameError::GameNotInMaintenance);
    Ok(())
}
