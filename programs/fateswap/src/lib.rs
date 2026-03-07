use anchor_lang::prelude::*;

declare_id!("EHYqhQQLKRy1Don3p57B3FozPM8TVHKip6tsSx9Nhp4k");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "FateSwap",
    project_url: "https://fateswap.io",
    contacts: "email:info@fateswap.io",
    policy: "https://fateswap.io/security",
    preferred_languages: "en",
    source_code: "https://github.com/fateswap-io/fateswap-program"
}

pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;
pub mod math;

use instructions::*;

#[program]
pub mod fateswap {
    use super::*;

    /// Initialize the ClearingHouse (LP pool)
    pub fn initialize(
        ctx: Context<Initialize>,
        fate_fee_bps: u16,
        max_bet_bps: u16,
        min_bet: u64,
        bet_timeout: i64,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, fate_fee_bps, max_bet_bps, min_bet, bet_timeout)
    }

    /// Deposit SOL and mint LP tokens
    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        instructions::deposit_sol::handler(ctx, amount)
    }

    /// Burn LP tokens and withdraw SOL
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, lp_amount: u64) -> Result<()> {
        instructions::withdraw_sol::handler(ctx, lp_amount)
    }

    /// Submit commitment hash for a player (settler only)
    pub fn submit_commitment(
        ctx: Context<SubmitCommitment>,
        player_key: Pubkey,
        commitment_hash: [u8; 32],
        nonce: u64,
    ) -> Result<()> {
        instructions::submit_commitment::handler(ctx, player_key, commitment_hash, nonce)
    }

    /// Place a fate order (bet)
    pub fn place_fate_order(
        ctx: Context<PlaceFateOrder>,
        nonce: u64,
        amount: u64,
        multiplier_bps: u32,
        token_mint: Pubkey,
        token_amount: u64,
    ) -> Result<()> {
        instructions::place_fate_order::handler(ctx, nonce, amount, multiplier_bps, token_mint, token_amount)
    }

    /// Settle a fate order (settler only)
    pub fn settle_fate_order(
        ctx: Context<SettleFateOrder>,
        filled: bool,
        server_seed: [u8; 32],
    ) -> Result<()> {
        instructions::settle_fate_order::handler(ctx, filled, server_seed)
    }

    /// Reclaim an expired order
    pub fn reclaim_expired_order(ctx: Context<ReclaimExpiredOrder>) -> Result<()> {
        instructions::reclaim_expired_order::handler(ctx)
    }

    /// Set referrer (one-time, with tier-2 auto-resolution)
    pub fn set_referrer(ctx: Context<SetReferrer>, referrer_key: Pubkey) -> Result<()> {
        instructions::set_referrer::handler(ctx, referrer_key)
    }

    /// Pause/unpause protocol (authority only)
    pub fn pause(ctx: Context<Pause>, paused: bool) -> Result<()> {
        instructions::pause::handler(ctx, paused)
    }

    /// Update configuration (authority only)
    pub fn update_config(
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
        instructions::update_config::handler(
            ctx,
            fate_fee_bps,
            max_bet_bps,
            min_bet,
            bet_timeout,
            referral_bps,
            tier2_referral_bps,
            nft_reward_bps,
            platform_fee_bps,
            bonus_bps,
            platform_wallet,
            bonus_wallet,
            nft_rewarder,
        )
    }

    /// Update settler wallet (authority only)
    pub fn update_settler(ctx: Context<UpdateSettler>) -> Result<()> {
        instructions::update_settler::handler(ctx)
    }

    /// Create Metaplex token metadata for the LP mint (authority only)
    pub fn create_lp_metadata(
        ctx: Context<CreateLpMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::create_lp_metadata::handler(ctx, name, symbol, uri)
    }
}
