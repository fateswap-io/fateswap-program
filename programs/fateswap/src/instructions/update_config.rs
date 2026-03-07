use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"clearing_house"],
        bump,
        has_one = authority
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    fate_fee_bps: Option<u16>,
    max_bet_bps: Option<u16>,
    min_bet: Option<u64>,
    bet_timeout: Option<i64>,
    referral_bps: Option<u16>,
    tier2_referral_bps: Option<u16>,
    nft_reward_bps: Option<u16>,
    platform_fee_bps: Option<u16>,
    bonus_bps: Option<u16>,
    platform_wallet: Option<Pubkey>,
    bonus_wallet: Option<Pubkey>,
    nft_rewarder: Option<Pubkey>,
) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    // Field IDs for events
    // 0=fate_fee_bps, 1=max_bet_bps, 2=min_bet, 3=bet_timeout,
    // 4=referral_bps, 5=tier2_referral_bps, 6=nft_reward_bps,
    // 7=platform_fee_bps, 8=bonus_bps

    if let Some(val) = fate_fee_bps {
        require!(val <= 1000, FateSwapError::InvalidConfig); // max 10%
        let old = clearing_house.fate_fee_bps;
        clearing_house.fate_fee_bps = val;
        emit!(ConfigUpdated {
            field_id: 0,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = max_bet_bps {
        require!(val <= 500, FateSwapError::InvalidConfig); // max 5%
        let old = clearing_house.max_bet_bps;
        clearing_house.max_bet_bps = val;
        emit!(ConfigUpdated {
            field_id: 1,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = min_bet {
        require!(val > 0, FateSwapError::InvalidConfig);
        let old = clearing_house.min_bet;
        clearing_house.min_bet = val;
        emit!(ConfigUpdated {
            field_id: 2,
            old_value: old,
            new_value: val,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = bet_timeout {
        require!(val >= 60, FateSwapError::InvalidConfig); // min 60 seconds
        let old = clearing_house.bet_timeout;
        clearing_house.bet_timeout = val;
        emit!(ConfigUpdated {
            field_id: 3,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = referral_bps {
        require!(val <= 100, FateSwapError::InvalidConfig); // max 1%
        let old = clearing_house.referral_bps;
        clearing_house.referral_bps = val;
        emit!(ConfigUpdated {
            field_id: 4,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = tier2_referral_bps {
        require!(val <= 100, FateSwapError::InvalidConfig); // max 1%
        let old = clearing_house.tier2_referral_bps;
        clearing_house.tier2_referral_bps = val;
        emit!(ConfigUpdated {
            field_id: 5,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = nft_reward_bps {
        require!(val <= 100, FateSwapError::InvalidConfig); // max 1%
        let old = clearing_house.nft_reward_bps;
        clearing_house.nft_reward_bps = val;
        emit!(ConfigUpdated {
            field_id: 6,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = platform_fee_bps {
        require!(val <= 100, FateSwapError::InvalidConfig); // max 1%
        let old = clearing_house.platform_fee_bps;
        clearing_house.platform_fee_bps = val;
        emit!(ConfigUpdated {
            field_id: 7,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    if let Some(val) = bonus_bps {
        require!(val <= 100, FateSwapError::InvalidConfig); // max 1%
        let old = clearing_house.bonus_bps;
        clearing_house.bonus_bps = val;
        emit!(ConfigUpdated {
            field_id: 8,
            old_value: old as u64,
            new_value: val as u64,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });
    }

    // Validate combined fee split doesn't exceed 5% (500 bps)
    let total_split = clearing_house.referral_bps as u32
        + clearing_house.tier2_referral_bps as u32
        + clearing_house.nft_reward_bps as u32
        + clearing_house.platform_fee_bps as u32
        + clearing_house.bonus_bps as u32;
    require!(total_split <= 500, FateSwapError::InvalidConfig);

    if let Some(val) = platform_wallet {
        clearing_house.platform_wallet = val;
    }

    if let Some(val) = bonus_wallet {
        clearing_house.bonus_wallet = val;
    }

    if let Some(val) = nft_rewarder {
        clearing_house.nft_rewarder = val;
    }

    Ok(())
}
