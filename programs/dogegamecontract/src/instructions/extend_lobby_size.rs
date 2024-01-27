use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction::transfer},
};
use anchor_spl::token::Mint;
use doge_o::get_authority;

use crate::{constants::lobby_account_size, state::LobbyState, utils::get_dtrk_mint};

#[derive(Accounts, Clone)]
pub struct ExtendLobbySize<'info> {
    #[account(mut, address = get_authority())]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"lobby", 
            track_holder.key().as_ref(),
            track_mint.key().as_ref(),
        ],
        bump = lobby_account.bump
    )]
    pub lobby_account: Account<'info, LobbyState>,
    pub track_holder: SystemAccount<'info>,
    pub track_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ExtendLobbySize>) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let lobby_account = &mut ctx.accounts.lobby_account;
    let lobby_account_info = lobby_account.to_account_info();

    let lobby_data_size = lobby_account_info.data.borrow().len();
    msg!("Lobby data size {}", lobby_data_size);

    if lobby_data_size < 8 + lobby_account_size(lobby_account.max_players) {
        let new_lobby_size = lobby_data_size + 32;
        let rent = Rent::get()?;
        let lamports_to_send = rent.minimum_balance(new_lobby_size) - lobby_account_info.lamports();

        invoke(
            &transfer(&authority.key(), &lobby_account.key(), lamports_to_send),
            &[
                authority.to_account_info(),
                lobby_account_info,
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        lobby_account
            .to_account_info()
            .realloc(new_lobby_size, false)?;
    } else {
        lobby_account.track_keys.entry_fee_mint = get_dtrk_mint();
    }

    Ok(())
}
