use crate::metadata::Metadata;
use crate::state::{
    EntryFeeRequirements, EntryFeeRequirementsState, LobbyData, LobbyState, TrackKeys,
};
use crate::utils::*;
use crate::{constants::*, error::GameError};
use anchor_lang::{prelude::*, solana_program::program_pack::IsInitialized};
use anchor_spl::associated_token::{create, get_associated_token_address, Create};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(lobby_data: LobbyData, max_players: u8)]
pub struct CreateLobby<'info> {
    #[account(mut)]
    pub track_holder: Signer<'info>,

    // Track state account
    #[account(
        init,
        space = 8 + lobby_account_size(max_players),
        payer = track_holder,
        seeds = [
            b"lobby", 
            track_holder.key().as_ref(),
            track_mint.key().as_ref(),
        ],
        bump
    )]
    pub lobby_account: Account<'info, LobbyState>,

    // Track and Entry fee token related accounts
    #[account(
        constraint = track_mint.is_initialized @ ErrorCode::AccountNotInitialized
    )]
    pub track_mint: Box<Account<'info, Mint>>,
    #[account(constraint = track_metadata.mint.eq(&track_mint.key()))]
    pub track_metadata: Account<'info, Metadata>,

    #[account(
        mut,
        constraint = track_holder_token.owner.eq(&track_holder.key()),
        constraint = track_holder_token.mint.eq(&track_mint.key()),
        constraint = track_holder_token.amount == 1,
    )]
    pub track_holder_token: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            b"entryfeerequirements",
            entry_fee_mint.key().as_ref(),
        ],
        bump = entry_fee_requirements_account.bump
    )]
    pub entry_fee_requirements_account: Box<Account<'info, EntryFeeRequirementsState>>,

    #[account(
        address = entry_fee_requirements_account.entry_fee_mint @ GameError::InvalidEntryFeeMint
    )]
    pub entry_fee_mint: Box<Account<'info, Mint>>,

    #[account(address = get_wsol_mint())]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = track_holder_entry_fee_token.owner.eq(&track_holder.key()),
        constraint = track_holder_entry_fee_token.mint.eq(&entry_fee_mint.key()),
        constraint = track_holder_entry_fee_token.is_initialized()
    )]
    pub track_holder_entry_fee_token: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Proper PDA validation is done
    pub lobby_entry_fee_token: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Proper PDA validation is done
    pub lobby_track_token: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Proper PDA validation is done
    pub lobby_wsol_token: UncheckedAccount<'info>,

    // Sysvar
    pub rent: Sysvar<'info, Rent>,

    // Program Accounts
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLobby<'info> {
    pub fn validate_init_lobby(
        &self,
        lobby_metadata: &LobbyData,
        entry_fee_requirements: &EntryFeeRequirements,
    ) -> Result<()> {
        require!(
            is_metadata_valid(&self.track_metadata.data),
            GameError::InvalidTrack
        );
        require!(
            is_lobby_metadata_valid(lobby_metadata, entry_fee_requirements)?,
            GameError::InvalidLobbyMetadata
        );

        let lobby_entry_fee_token =
            get_associated_token_address(&self.lobby_account.key(), &self.entry_fee_mint.key());

        let lobby_track_address =
            get_associated_token_address(&self.lobby_account.key(), &self.track_mint.key());

        require!(
            lobby_entry_fee_token.eq(&self.lobby_entry_fee_token.key()),
            StateInvalidAddress
        );

        require!(
            lobby_track_address.eq(&self.lobby_track_token.key()),
            StateInvalidAddress
        );

        Ok(())
    }

    pub fn create_token_ctx(
        &self,
        mint: &Account<'info, Mint>,
        token_account: &UncheckedAccount<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Create<'info>> {
        CpiContext::new(
            self.associated_token_program.to_account_info(),
            Create {
                payer: self.track_holder.to_account_info(),
                associated_token: token_account.to_account_info(),
                authority: self.lobby_account.to_account_info(),
                mint: mint.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }

    pub fn track_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.track_holder_token.to_account_info(),
                to: self.lobby_track_token.to_account_info(),
                authority: self.track_holder.to_account_info(),
            },
        )
    }
}

pub fn handler(
    ctx: Context<CreateLobby>,
    lobby_metadata: LobbyData,
    max_players: u8,
) -> Result<()> {
    //require!(!MAINTENANCE_MODE, GameError::GameInMaintenance);
    require!(max_players >= 2, GameError::InvalidMaxPlayersCount);

    let entry_fee_requirements = &ctx
        .accounts
        .entry_fee_requirements_account
        .entry_fee_requirements;

    ctx.accounts
        .validate_init_lobby(&lobby_metadata, &entry_fee_requirements)?;

    let entry_fee_mint = &ctx.accounts.entry_fee_mint;
    let wsol_mint = &ctx.accounts.wsol_mint;
    let track_mint = &ctx.accounts.track_mint;

    let lobby_track_token = &ctx.accounts.lobby_track_token;
    let lobby_wsol_token = &ctx.accounts.lobby_wsol_token;
    let lobby_entry_fee_token = &ctx.accounts.lobby_entry_fee_token;

    msg!("Creating Lobby entry fee token vault");
    create(
        ctx.accounts
            .create_token_ctx(entry_fee_mint, lobby_entry_fee_token),
    )?;

    if entry_fee_mint.key() != get_wsol_mint() {
        msg!("Creating Lobby WSOL vault");
        create(ctx.accounts.create_token_ctx(wsol_mint, lobby_wsol_token))?;
    }

    msg!("Creating Lobby Track vault");
    create(ctx.accounts.create_token_ctx(track_mint, lobby_track_token))?;

    msg!("Transfer Track NFT");
    transfer(ctx.accounts.track_transfer_ctx(), 1)?;

    let current_timestamp = Clock::get()?.unix_timestamp;

    msg!(
        "Creating Lobby State Account. Max players are set to {}",
        max_players
    );

    let lobby_state_account = &mut ctx.accounts.lobby_account;

    lobby_state_account.bump = *ctx.bumps.get("lobby_account").unwrap();
    lobby_state_account.max_players = max_players;
    lobby_state_account.race_started = false;
    lobby_state_account.lobby_data = lobby_metadata;

    lobby_state_account.racers = fill_empty_racers(max_players);
    lobby_state_account.track_keys = TrackKeys {
        entry_fee_mint: ctx.accounts.entry_fee_mint.key(),
        lobby_entry_fee_token: ctx.accounts.lobby_entry_fee_token.key(),
        lobby_track_token: ctx.accounts.lobby_track_token.key(),
        track_holder: ctx.accounts.track_holder.key(),
        track_holder_entry_fee_token: ctx.accounts.track_holder_entry_fee_token.key(),
        track_holder_token: ctx.accounts.track_holder_token.key(),
        track_metadata: ctx.accounts.track_metadata.key(),
        track_mint: ctx.accounts.track_mint.key(),
        lobby_wsol_token: ctx.accounts.lobby_wsol_token.key(),
    };
    let lobby_unlock_time = (current_timestamp + COOLDOWN_PERIOD) as u64;
    lobby_state_account.unlock_time = lobby_unlock_time;

    msg!("Lobby Unlock at {}", lobby_unlock_time);

    Ok(())
}
