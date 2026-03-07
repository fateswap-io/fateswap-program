use anchor_lang::prelude::*;

// ============================================================================
// Phase S1: ClearingHouse Events
// ============================================================================

#[event]
pub struct ClearingHouseInitialized {
    pub authority: Pubkey,
    pub settler: Pubkey,
    pub vault: Pubkey,
    pub lp_mint: Pubkey,
    pub fate_fee_bps: u16,
    pub max_bet_bps: u16,
    pub min_bet: u64,
    pub bet_timeout: i64,
    pub platform_wallet: Pubkey,
    pub bonus_wallet: Pubkey,
    pub nft_rewarder: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityDeposited {
    pub depositor: Pubkey,
    pub sol_amount: u64,
    pub lp_amount: u64,
    pub vault_balance: u64,
    pub lp_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityWithdrawn {
    pub withdrawer: Pubkey,
    pub lp_amount: u64,
    pub sol_amount: u64,
    pub vault_balance: u64,
    pub lp_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct Paused {
    pub authority: Pubkey,
    pub paused: bool,
    pub timestamp: i64,
}

// ============================================================================
// Phase S2: FateGame Events
// ============================================================================

#[event]
pub struct CommitmentSubmitted {
    pub player: Pubkey,
    pub commitment_hash: [u8; 32],
    pub nonce: u64,
    pub timestamp: i64,
}

#[event]
pub struct FateOrderPlaced {
    pub player: Pubkey,
    pub order: Pubkey,
    pub amount: u64,
    pub multiplier_bps: u32,
    pub potential_payout: u64,
    pub commitment_hash: [u8; 32],
    pub nonce: u64,
    pub token_mint: Pubkey,
    pub token_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct FateOrderSettled {
    pub player: Pubkey,
    pub order: Pubkey,
    pub filled: bool,
    pub amount: u64,
    pub multiplier_bps: u32,
    pub payout: u64,
    pub server_seed: [u8; 32],
    pub commitment_hash: [u8; 32],
    pub nonce: u64,
    pub timestamp: i64,
}

#[event]
pub struct FateOrderReclaimed {
    pub player: Pubkey,
    pub order: Pubkey,
    pub refund_amount: u64,
    pub timestamp: i64,
}

// ============================================================================
// Phase S3: Referral Events
// ============================================================================

#[event]
pub struct ReferrerSet {
    pub player: Pubkey,
    pub referrer: Pubkey,
    pub tier2_referrer: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardPaid {
    pub reward_type: u8, // 0=tier1_referral, 1=tier2_referral, 2=nft_reward, 3=platform_fee, 4=bonus
    pub recipient: Pubkey,
    pub amount: u64,
    pub order: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardFailed {
    pub reward_type: u8,
    pub recipient: Pubkey,
    pub amount: u64,
    pub order: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ConfigUpdated {
    pub field_id: u8, // 0=fate_fee_bps, 1=max_bet_bps, 2=min_bet, 3=bet_timeout, 4=referral_bps, 5=tier2_referral_bps, 6=nft_reward_bps, 7=platform_fee_bps, 8=bonus_bps
    pub old_value: u64,
    pub new_value: u64,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SettlerUpdated {
    pub old_settler: Pubkey,
    pub new_settler: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}
