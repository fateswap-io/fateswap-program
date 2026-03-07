use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ClearingHouseState::LEN,
        seeds = [b"clearing_house"],
        bump
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    #[account(
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: Vault PDA that holds SOL
    pub vault: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"lp_mint"],
        bump,
        mint::decimals = 9,
        mint::authority = lp_authority,
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"lp_authority"],
        bump
    )]
    /// CHECK: LP mint authority PDA
    pub lp_authority: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Settler wallet (can be any pubkey)
    pub settler: AccountInfo<'info>,

    /// CHECK: Platform wallet for platform fees
    pub platform_wallet: AccountInfo<'info>,

    /// CHECK: Bonus wallet for bonus pool
    pub bonus_wallet: AccountInfo<'info>,

    /// CHECK: NFT rewarder program/vault
    pub nft_rewarder: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<Initialize>,
    fate_fee_bps: u16,
    max_bet_bps: u16,
    min_bet: u64,
    bet_timeout: i64,
) -> Result<()> {
    // Validate config bounds
    require!(fate_fee_bps <= 1000, FateSwapError::InvalidConfig); // max 10%
    require!(max_bet_bps <= 500, FateSwapError::InvalidConfig);   // max 5%
    require!(min_bet > 0, FateSwapError::InvalidConfig);
    require!(bet_timeout >= 60, FateSwapError::InvalidConfig);    // min 60 seconds

    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    // Store accounts
    clearing_house.authority = ctx.accounts.authority.key();
    clearing_house.settler = ctx.accounts.settler.key();
    clearing_house.vault = ctx.accounts.vault.key();
    clearing_house.lp_mint = ctx.accounts.lp_mint.key();
    clearing_house.lp_authority = ctx.accounts.lp_authority.key();

    // Store bumps
    clearing_house.vault_bump = ctx.bumps.vault;
    clearing_house.lp_mint_bump = ctx.bumps.lp_mint;
    clearing_house.lp_authority_bump = ctx.bumps.lp_authority;

    // Store config
    clearing_house.paused = false;
    clearing_house.fate_fee_bps = fate_fee_bps;
    clearing_house.max_bet_bps = max_bet_bps;
    clearing_house.min_bet = min_bet;
    clearing_house.bet_timeout = bet_timeout;

    // Initialize 5-way split to zero (will be configured later via update_config)
    clearing_house.referral_bps = 0;
    clearing_house.tier2_referral_bps = 0;
    clearing_house.nft_reward_bps = 0;
    clearing_house.platform_fee_bps = 0;
    clearing_house.bonus_bps = 0;

    // Store split wallets
    clearing_house.platform_wallet = ctx.accounts.platform_wallet.key();
    clearing_house.bonus_wallet = ctx.accounts.bonus_wallet.key();
    clearing_house.nft_rewarder = ctx.accounts.nft_rewarder.key();

    // Initialize statistics
    clearing_house.total_liability = 0;
    clearing_house.unsettled_count = 0;
    clearing_house.total_bets = 0;
    clearing_house.total_filled = 0;
    clearing_house.total_not_filled = 0;
    clearing_house.total_volume = 0;
    clearing_house.house_profit = 0;

    // Initialize trading accounting
    clearing_house.total_payout = 0;
    clearing_house.largest_bet = 0;
    clearing_house.largest_payout = 0;

    // Initialize fee tracking
    clearing_house.total_referral_paid = 0;
    clearing_house.total_nft_rewards_paid = 0;
    clearing_house.total_platform_fees_paid = 0;
    clearing_house.total_bonus_paid = 0;

    // Initialize LP activity
    clearing_house.total_deposited = 0;
    clearing_house.total_withdrawn = 0;
    clearing_house.lp_deposits_count = 0;
    clearing_house.lp_withdrawals_count = 0;

    // Initialize reserved space
    clearing_house._reserved = [0; 104];

    // Emit event
    emit!(ClearingHouseInitialized {
        authority: ctx.accounts.authority.key(),
        settler: ctx.accounts.settler.key(),
        vault: ctx.accounts.vault.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        fate_fee_bps,
        max_bet_bps,
        min_bet,
        bet_timeout,
        platform_wallet: ctx.accounts.platform_wallet.key(),
        bonus_wallet: ctx.accounts.bonus_wallet.key(),
        nft_rewarder: ctx.accounts.nft_rewarder.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
