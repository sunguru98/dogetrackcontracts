use crate::{constants::*, error::GameError, state::EntryFeeRequirements, utils::get_wsol_mint};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use doge_o::get_authority;

use crate::state::EntryFeeRequirementsState;

#[derive(Accounts)]
pub struct InitEntryFeeRequirements<'info> {
    #[account(
        mut,
        address = get_authority()
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + ENTRY_FEE_REQUIREMENT_SIZE,
        seeds = [
            b"entryfeerequirements",
            entry_fee_mint.key().as_ref(),
        ],
        bump
    )]
    pub entry_fee_requirements_account: Account<'info, EntryFeeRequirementsState>,
    pub entry_fee_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitEntryFeeRequirements>,
    EntryFeeRequirements {
        min_fee,
        max_class_1_fee,
        max_class_2_fee,
        max_class_3_fee,
        max_class_4_fee,
        max_class_5_fee,
    }: EntryFeeRequirements,
) -> Result<()> {
    let entry_fee_requirements_state = &mut ctx.accounts.entry_fee_requirements_account;
    let entry_fee_mint = &ctx.accounts.entry_fee_mint;

    if entry_fee_mint.key().ne(&get_wsol_mint()) && entry_fee_mint.supply <= 1 {
        return err!(GameError::CannotAcceptNFTAsEntryFee);
    }

    entry_fee_requirements_state.bump = *ctx.bumps.get("entry_fee_requirements_account").unwrap();
    entry_fee_requirements_state.entry_fee_mint = entry_fee_mint.key();

    require!(min_fee > 0, GameError::InvalidEntryFeeRequirement);
    entry_fee_requirements_state.entry_fee_requirements.min_fee = min_fee;

    require!(
        max_class_1_fee > min_fee,
        GameError::InvalidEntryFeeRequirement
    );
    entry_fee_requirements_state
        .entry_fee_requirements
        .max_class_1_fee = max_class_1_fee;

    require!(
        max_class_2_fee > max_class_1_fee,
        GameError::InvalidEntryFeeRequirement
    );
    entry_fee_requirements_state
        .entry_fee_requirements
        .max_class_2_fee = max_class_2_fee;

    require!(
        max_class_3_fee > max_class_2_fee,
        GameError::InvalidEntryFeeRequirement
    );
    entry_fee_requirements_state
        .entry_fee_requirements
        .max_class_3_fee = max_class_3_fee;

    require!(
        max_class_4_fee > max_class_3_fee,
        GameError::InvalidEntryFeeRequirement
    );
    entry_fee_requirements_state
        .entry_fee_requirements
        .max_class_4_fee = max_class_4_fee;

    require!(
        max_class_5_fee > max_class_4_fee,
        GameError::InvalidEntryFeeRequirement
    );
    entry_fee_requirements_state
        .entry_fee_requirements
        .max_class_5_fee = max_class_5_fee;

    Ok(())
}
