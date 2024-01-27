use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

pub mod errors;
pub mod metadata;
pub mod stats;

use crate::errors::*;
use crate::metadata::Metadata;
use crate::stats::*;
use anchor_spl::token::Burn;
use anchor_spl::token::{Mint, TokenAccount};
use mpl_token_metadata::state::PREFIX;

declare_id!("BQmZuoj4q3gU7Qzqi3siJ4R48Tj9uua3AMDhqTDNDdFu");

pub const DTRK_MINT_ADDRESS: &str = "DTRK1XRNaL6CxfFWwVLZMxyhiCZnwdP32CgzVDXWy5Td";

pub const DOGE_VERIFIED_CREATOR: &str = "SCL4YPcMCXbWHGPvYxR1e2c7UMFJEZNz88Wr3U8Etj6";

pub const AUTHORITY: &str = "4YH9irnaCnm12PvSbsssYLxkPGuLS2Ho3q3aehm847xe";

pub const MAINTENANCE_MODE: bool = false;

pub const DOGE_STATS_SIZE: usize = 8 + 1 + 1 + 1 + 1 + 1 + 32 + 32 + 32;

pub fn get_dtrk() -> Pubkey {
    Pubkey::from_str(DTRK_MINT_ADDRESS).unwrap()
}

pub fn get_verified_creator() -> Pubkey {
    Pubkey::from_str(DOGE_VERIFIED_CREATOR).unwrap()
}

pub fn get_authority() -> Pubkey {
    Pubkey::from_str(AUTHORITY).unwrap()
}

#[program]
pub mod doge_o {

    use super::*;

    pub fn init_stats(ctx: Context<InitStats>, stats: Stats) -> Result<()> {
        ctx.accounts.validate_doge_metadata()?;

        require!(stats.validate(), DogeError::InvalidStats);

        msg!(
            "Initializing Doge Stats for mint {}",
            &ctx.accounts.doge_mint.key().to_string()
        );

        msg!("Endurance: {}", stats.endurance);
        msg!("Speed: {}", stats.speed);
        msg!("Agility: {}", stats.agility);

        let doge_stats = &mut ctx.accounts.doge_stats;

        doge_stats.agility = stats.agility;
        doge_stats.endurance = stats.endurance;
        doge_stats.speed = stats.speed;
        doge_stats.win_percentage = 0;
        doge_stats.doge_bump = *ctx.bumps.get("doge_stats").unwrap();
        doge_stats.doge_mint = ctx.accounts.doge_mint.key();
        doge_stats.doge_metadata = ctx.accounts.doge_metadata.key();
        doge_stats.init_authority = ctx.accounts.init_authority.key();

        Ok(())
    }

    pub fn upgrade_doge(ctx: Context<UpgradeDoge>, new_stats: Stats) -> Result<()> {
        require!(!MAINTENANCE_MODE, DogeError::MaintenanceMode);

        let old_stats = Stats {
            speed: ctx.accounts.doge_stats.speed,
            endurance: ctx.accounts.doge_stats.endurance,
            agility: ctx.accounts.doge_stats.agility,
        };

        msg!(
            "Old stats Endurance {} Speed {} Agility {}",
            old_stats.endurance,
            old_stats.speed,
            old_stats.agility
        );

        // Validating stats between 1 - 100
        require!(new_stats.validate(), DogeError::InvalidUpdateStats);

        // Cross checking costs
        let fees = old_stats.calculate_cost(&new_stats)?;
        let fees_dtrk = 10u64
            .checked_pow((ctx.accounts.dtrk_mint.decimals) as u32)
            .and_then(|res| res.checked_mul(fees))
            .ok_or(DogeError::InvalidUpdateStats)?;

        let token_balance = ctx.accounts.dtrk_token.amount;
        require!(token_balance >= fees_dtrk, DogeError::InsufficientTokens);

        if fees > 0 {
            msg!("Required DTRK for upgrade {}", fees);
            token::burn(ctx.accounts.token_burn_context(), fees_dtrk)?;

            msg!(
                "Upgrading for doge {}",
                ctx.accounts.doge_mint.key().to_string()
            );

            let doge_stats = &mut ctx.accounts.doge_stats;

            doge_stats.endurance = new_stats.endurance;
            doge_stats.speed = new_stats.speed;
            doge_stats.agility = new_stats.agility;

            msg!(
                "New Endurance {} Agility {} Speed {}",
                doge_stats.endurance,
                doge_stats.agility,
                doge_stats.speed
            );
        }

        Ok(())
    }

    pub fn set_win_percentage(ctx: Context<SetWinPercentage>, win_pct: u8) -> Result<()> {
        require!(!MAINTENANCE_MODE, DogeError::MaintenanceMode);
        let doge_o_stats = &mut ctx.accounts.doge_stats;
        require!(win_pct <= 100, DogeError::InvalidWinPercentage);
        doge_o_stats.win_percentage = win_pct;
        Ok(())
    }

    pub fn close_old_accounts(ctx: Context<CloseOldAccounts>) -> Result<()> {
        let doge_stats_account = &mut ctx.accounts.doge_stats;
        doge_stats_account.doge_bump = 0;
        doge_stats_account.agility = 0;
        doge_stats_account.speed = 0;
        doge_stats_account.endurance = 0;
        doge_stats_account.win_percentage = 0;
        doge_stats_account.doge_metadata = Pubkey::default();
        doge_stats_account.doge_mint = Pubkey::default();
        doge_stats_account.init_authority = Pubkey::default();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseOldAccounts<'info> {
    #[account(mut, address = get_authority())]
    authority: Signer<'info>,
    #[account(mut)]
    destination: SystemAccount<'info>,
    init_authority: SystemAccount<'info>,
    #[account(
        mut,
        close = destination,
        seeds = [b"dogeo", init_authority.key().as_ref(), doge_mint.key().as_ref()], 
        bump
    )]
    doge_stats: Account<'info, DogeStats>,
    doge_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct InitStats<'info> {
    #[account(mut, address = get_authority())]
    pub init_authority: Signer<'info>,
    #[account(
        init,
        payer = init_authority,
        space = DOGE_STATS_SIZE,
        seeds = [b"dogeo", init_authority.key().as_ref(), doge_mint.key().as_ref()],
        bump
    )]
    pub doge_stats: Account<'info, DogeStats>,
    pub doge_mint: Account<'info, Mint>,
    #[account(
        constraint = doge_metadata.update_authority.eq(&init_authority.key()),
        constraint = doge_metadata.mint.eq(&doge_mint.key())
    )]
    pub doge_metadata: Account<'info, Metadata>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitStats<'info> {
    pub fn new_stats(&self, speed: u8, agility: u8, endurance: u8) -> Stats {
        Stats {
            speed,
            agility,
            endurance,
        }
    }

    pub fn validate_doge_metadata(&self) -> Result<()> {
        let (expected_metadata, _) = Pubkey::find_program_address(
            &[
                PREFIX.as_bytes(),
                Metadata::owner().as_ref(),
                self.doge_mint.key().as_ref(),
            ],
            &Metadata::owner(),
        );

        if self.doge_metadata.key().ne(&expected_metadata) {
            return err!(DogeError::InvalidDogeMetadata);
        }

        if let Some(creators) = &self.doge_metadata.data.creators {
            let verified_creator = creators
                .iter()
                .find(|&creator| creator.address.eq(&get_verified_creator()) && creator.verified);

            if verified_creator.is_none() {
                return err!(DogeError::InvalidDogeMetadata);
            }

            Ok(())
        } else {
            return err!(DogeError::InvalidDogeMetadata);
        }
    }
}

#[derive(Accounts)]
pub struct UpgradeDoge<'info> {
    #[account(mut,
        has_one = init_authority,
        has_one = doge_mint,
        has_one = doge_metadata,
        seeds = [b"dogeo", init_authority.key().as_ref(), doge_mint.key().as_ref()], 
        bump = doge_stats.doge_bump
    )]
    pub doge_stats: Account<'info, DogeStats>,
    #[account(
        mut,
        constraint = dtrk_token.owner.eq(&doge_holder.key()),
        constraint = dtrk_token.mint.eq(&dtrk_mint.key()),
    )]
    pub dtrk_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = dtrk_mint.key().eq(&get_dtrk())
    )]
    pub dtrk_mint: Account<'info, Mint>,
    pub doge_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = doge_token.mint.eq(&doge_mint.key()),
        constraint = doge_token.owner.eq(&doge_holder.key()),
        constraint = doge_token.amount.eq(&1) @ DogeError::EmptyDogeToken
    )]
    pub doge_token: Box<Account<'info, TokenAccount>>,
    pub doge_metadata: Box<Account<'info, Metadata>>,
    pub init_authority: SystemAccount<'info>,
    #[account(mut)]
    pub doge_holder: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> UpgradeDoge<'info> {
    pub fn token_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.dtrk_mint.to_account_info(),
                from: self.dtrk_token.to_account_info(),
                authority: self.doge_holder.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct SetWinPercentage<'info> {
    #[account(mut, address = get_authority())]
    pub authority: Signer<'info>,
    pub init_authority: SystemAccount<'info>,
    #[account(mut,
        has_one = init_authority,
        has_one = doge_mint,
        seeds = [b"dogeo", init_authority.key().as_ref(), doge_mint.key().as_ref()], 
        bump = doge_stats.doge_bump
    )]
    pub doge_stats: Account<'info, DogeStats>,
    pub doge_mint: Account<'info, Mint>,
}

#[account]
#[derive(Default)]
pub struct DogeStats {
    pub agility: u8,
    pub speed: u8,
    pub endurance: u8,
    pub doge_bump: u8,
    pub win_percentage: u8,
    pub doge_mint: Pubkey,
    pub init_authority: Pubkey,
    pub doge_metadata: Pubkey,
}
