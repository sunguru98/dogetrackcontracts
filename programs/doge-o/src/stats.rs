use anchor_lang::prelude::*;

use crate::errors::DogeError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct Stats {
    pub endurance: u8,
    pub speed: u8,
    pub agility: u8,
}

impl Stats {
    pub fn validate(&self) -> bool {
        self.agility > 0
            && self.agility <= 100
            && self.endurance > 0
            && self.endurance <= 100
            && self.speed > 0
            && self.speed <= 100
    }

    pub fn calculate_cost(&self, new_stats: &Stats) -> Result<u64> {
        if !self.validate_new_stats(new_stats) {
            msg!("Error: Invalid new stats");
            return err!(DogeError::InvalidUpdateStats);
        }

        let total_fees = calculate_dtrk(new_stats.endurance, self.endurance)?
            + calculate_dtrk(new_stats.agility, self.agility)?
            + calculate_dtrk(new_stats.speed, self.speed)?;

        Ok(total_fees)
    }

    pub fn validate_new_stats(&self, new_stats: &Stats) -> bool {
        self.endurance <= new_stats.endurance
            && self.agility <= new_stats.agility
            && self.speed <= new_stats.speed
    }
}

fn calculate_dtrk(new_stat: u8, old_stat: u8) -> Result<u64> {
    let mut old_stat = old_stat;
    let mut fee_level = fetch_fee_level(old_stat);

    let mut level_diff = new_stat
        .checked_sub(old_stat)
        .ok_or(DogeError::InvalidUpdateStats)?;

    let mut fees: u64 = 0;

    while level_diff != 0 {
        let nearest_checkpoint = fetch_nearest_checkpoint(old_stat);
        let limit = if fetch_nearest_checkpoint(old_stat).gt(&new_stat) {
            new_stat
        } else {
            nearest_checkpoint
        };

        let limit_diff = limit
            .checked_sub(old_stat)
            .ok_or(DogeError::InvalidUpdateStats)?;

        let floor_level_diff = if (limit_diff).lt(&20) { limit_diff } else { 20 };

        if floor_level_diff == 0u8 && old_stat == nearest_checkpoint {
            fees = fees
                .checked_add(fetch_fee_level(old_stat + 1) as u64)
                .ok_or(DogeError::InvalidUpdateStats)?;
        } else {
            fees = fees
                .checked_add(fee_level as u64 * floor_level_diff as u64)
                .ok_or(DogeError::InvalidUpdateStats)?;
        }

        let conc_floor_level_diff = if floor_level_diff.eq(&0) {
            1
        } else {
            floor_level_diff
        };

        old_stat = old_stat
            .checked_add(conc_floor_level_diff)
            .ok_or(DogeError::InvalidUpdateStats)?;

        fee_level = fetch_fee_level(old_stat);
        level_diff = level_diff
            .checked_sub(conc_floor_level_diff)
            .ok_or(DogeError::InvalidUpdateStats)?;
    }

    Ok(fees)
}

fn fetch_fee_level(stat: u8) -> u8 {
    if stat > 1 && stat <= 20 {
        18u8
    } else if (21..=40).contains(&stat) {
        36u8
    } else if (41..=60).contains(&stat) {
        72u8
    } else if (61..=80).contains(&stat) {
        144u8
    } else if (81..=100).contains(&stat) {
        255u8
    } else {
        0u8
    }
}

fn fetch_nearest_checkpoint(stat: u8) -> u8 {
    if stat > 1 && stat <= 20 {
        20u8
    } else if stat > 20 && stat <= 40 {
        40u8
    } else if stat > 40 && stat <= 60 {
        60u8
    } else if stat > 60 && stat <= 80 {
        80u8
    } else if stat > 80 && stat <= 100 {
        100u8
    } else {
        0u8
    }
}
