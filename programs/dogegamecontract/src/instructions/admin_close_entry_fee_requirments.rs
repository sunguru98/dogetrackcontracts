use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use doge_o::get_authority;

use crate::state::EntryFeeRequirementsState;

#[derive(Accounts)]
pub struct AdminCloseEntryFeeRequirements<'info> {
    #[account(
        address = get_authority()
    )]
    pub authority: Signer<'info>,

    pub entry_fee_mint: Account<'info, Mint>,

    #[account(
        mut,
        close = authority
    )]
    pub entry_fee_requirements_account: Account<'info, EntryFeeRequirementsState>,
}

pub fn handler(_: Context<AdminCloseEntryFeeRequirements>) -> Result<()> {
    //require!(MAINTENANCE_MODE, GameError::GameNotInMaintenance);
    Ok(())
}
