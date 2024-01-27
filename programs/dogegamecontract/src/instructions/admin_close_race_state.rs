use anchor_lang::prelude::*;
use doge_o::get_authority;

use crate::state::RaceState;

#[derive(Accounts)]
#[instruction(race_started: u64)]
pub struct AdminCloseRaceState<'info> {
    #[account(
        mut,
        address = get_authority()
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
    )]
    pub race_data_state: Account<'info, RaceState>,
}

pub fn handler(_: Context<AdminCloseRaceState>, _: u64) -> Result<()> {
    //require!(MAINTENANCE_MODE, GameError::GameNotInMaintenance);
    Ok(())
}
