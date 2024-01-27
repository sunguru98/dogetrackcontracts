use crate::{constants::*, state::EntryFeeRequirements, state::LobbyData};
use anchor_lang::{
    prelude::*,
    solana_program::{
        program_memory::sol_memcmp,
        program_pack::{IsInitialized, Pack},
    },
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::spl_token::{id as token_program_id, state::Account},
};
use mpl_token_metadata::state::Data;
use std::{result::Result as StdResult, str::FromStr};

pub fn get_track_creator() -> Pubkey {
    Pubkey::from_str(TRACK_VERIFIED_CREATOR).unwrap()
}

pub fn get_dtrk_mint() -> Pubkey {
    Pubkey::from_str(DTRK_MINT).unwrap()
}

pub fn get_wsol_mint() -> Pubkey {
    Pubkey::from_str(NATIVE_MINT).unwrap()
}

pub fn get_treasury_address() -> Pubkey {
    Pubkey::from_str(TREASURY_ADDRESS).unwrap()
}

pub fn is_doge_stats_valid(
    init_authority: &Pubkey,
    doge_mint: &Pubkey,
    doge_o_pda: &Pubkey,
) -> bool {
    let (doge_pda, _) = Pubkey::find_program_address(
        &[b"dogeo", init_authority.as_ref(), doge_mint.as_ref()],
        &doge_o::id(),
    );

    doge_pda.eq(doge_o_pda)
}

pub fn is_metadata_valid(data: &Data) -> bool {
    if let Some(ref creators) = data.creators {
        match creators
            .iter()
            .any(|creator| creator.address.eq(&get_track_creator()) && creator.verified)
        {
            true => data.name.contains("Genesis Track"),
            false => false,
        }
    } else {
        false
    }
}

pub fn fill_empty_racers(max_players: u8) -> Vec<Pubkey> {
    let mut players: Vec<Pubkey> = vec![];
    for _ in 0..max_players {
        players.push(Pubkey::default())
    }
    players
}

pub fn find_racer_index(racers: &[Pubkey], racer_to_find: Option<Pubkey>) -> Option<usize> {
    racers
        .iter()
        .position(|&racer| racer.eq(&racer_to_find.unwrap_or_default()))
}

pub fn percentage_of(value: u64, pct: u64) -> StdResult<u64, ProgramError> {
    value
        .checked_mul(pct)
        .and_then(|product| product.checked_div(100u64))
        .ok_or(ProgramError::InvalidAccountData)
}

pub fn is_lobby_empty(racers: &[Pubkey]) -> bool {
    racers.iter().any(|&racer| racer.eq(&Pubkey::default()))
}

pub fn is_lobby_metadata_valid(
    metadata: &LobbyData,
    entry_fee_requirements: &EntryFeeRequirements,
) -> Result<bool> {
    let LobbyData {
        entry_fee,
        location,
        min_class,
        name,
        total_laps,
        track_type: _,
    } = metadata;

    let (min_entry_fee, max_entry_fee) = get_entry_fee_bounds(min_class, &entry_fee_requirements);

    Ok(location.len().ge(&5)
        && location.len().le(&32)
        && name.len().ge(&5)
        && name.len().le(&32)
        && entry_fee.ge(&min_entry_fee)
        && entry_fee.le(&max_entry_fee)
        && min_class.ge(&1)
        && min_class.le(&5)
        && total_laps.ge(&1)
        && total_laps.lt(&5))
}

pub fn get_entry_fee_bounds(
    class: &u8,
    EntryFeeRequirements {
        min_fee,
        max_class_1_fee,
        max_class_2_fee,
        max_class_3_fee,
        max_class_4_fee,
        max_class_5_fee,
    }: &EntryFeeRequirements,
) -> (u64, u64) {
    if class.eq(&1u8) {
        return (*min_fee, *max_class_1_fee);
    } else if class.eq(&2u8) {
        return (*min_fee, *max_class_2_fee);
    } else if class.eq(&3u8) {
        return (*min_fee, *max_class_3_fee);
    } else if class.eq(&4u8) {
        return (*min_fee, *max_class_4_fee);
    } else if class.eq(&5u8) {
        return (*min_fee, *max_class_5_fee);
    } else {
        return (*min_fee, *max_class_1_fee);
    }
}

pub fn convert_to_entry_fee(amount: u64, decimals: u8) -> StdResult<u64, ProgramError> {
    amount
        .checked_mul(
            10u64
                .checked_pow(decimals.into())
                .ok_or(ProgramError::InvalidArgument)?,
        )
        .ok_or(ProgramError::InvalidArgument)
}

pub fn convert_to_entry_fee_without_decimals(
    amount_with_decimals: u64,
    decimals: u8,
) -> StdResult<u64, ProgramError> {
    amount_with_decimals
        .checked_div(
            10u64
                .checked_pow(decimals.into())
                .ok_or(ProgramError::InvalidArgument)?,
        )
        .ok_or(ProgramError::InvalidArgument)
}

pub fn check_valid_ata(
    token_account: &UncheckedAccount,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<()> {
    if token_account
        .key()
        .ne(&get_associated_token_address(owner, mint))
    {
        msg!("Invalid ATA address");
        return err!(ErrorCode::StateInvalidAddress);
    }

    if token_account.data_is_empty() {
        msg!("ATA empty");
        return err!(ErrorCode::AccountNotInitialized);
    }

    if token_account.owner.ne(&token_program_id()) {
        msg!("ATA program owner mismatch");
        return err!(ErrorCode::InvalidProgramId);
    }

    let account = Account::unpack_unchecked(&token_account.to_account_info().data.borrow())?;

    if !account.is_initialized() {
        msg!("ATA not initialized");
        return err!(ErrorCode::AccountNotInitialized);
    }

    if sol_memcmp(account.mint.as_ref(), mint.as_ref(), 32) != 0 {
        msg!("ATA mint mismatch");
        return err!(ErrorCode::ConstraintTokenMint);
    }

    if sol_memcmp(account.owner.as_ref(), owner.as_ref(), 32) != 0 {
        msg!("ATA owner mismatch");
        return err!(ErrorCode::ConstraintTokenOwner);
    }

    Ok(())
}
