use anchor_lang::prelude::*;

/// ClearingHouse state account (546 bytes including 8-byte discriminator)
/// Manages the LP pool, vault, and configuration for the FateSwap protocol
#[account]
pub struct ClearingHouseState {
    /// Authority that can update config and pause
    pub authority: Pubkey,              // 32

    /// Authorized settler wallet
    pub settler: Pubkey,                // 32

    /// Vault PDA that holds SOL
    pub vault: Pubkey,                  // 32

    /// LP token mint PDA
    pub lp_mint: Pubkey,                // 32

    /// LP mint authority PDA
    pub lp_authority: Pubkey,           // 32

    /// PDA bumps
    pub vault_bump: u8,                 // 1
    pub lp_mint_bump: u8,               // 1
    pub lp_authority_bump: u8,          // 1

    /// Protocol paused flag
    pub paused: bool,                   // 1

    /// Fee configuration (in basis points, 10000 = 100%)
    pub fate_fee_bps: u16,              // 2 - House edge (max 1000 = 10%)
    pub max_bet_bps: u16,               // 2 - Max bet as % of net balance (max 500 = 5%)
    pub min_bet: u64,                   // 8 - Minimum bet in lamports
    pub bet_timeout: i64,               // 8 - Timeout in seconds (min 60)

    /// Referral and revenue split configuration (in basis points)
    pub referral_bps: u16,              // 2 - Tier-1 referrer reward (max 100 = 1%)
    pub tier2_referral_bps: u16,        // 2 - Tier-2 referrer reward (max 100 = 1%)
    pub nft_reward_bps: u16,            // 2 - NFT holder rewards (max 100 = 1%)
    pub platform_fee_bps: u16,          // 2 - Platform fee (max 100 = 1%)
    pub bonus_bps: u16,                 // 2 - Bonus pool (max 100 = 1%)

    /// Wallets for revenue split
    pub platform_wallet: Pubkey,        // 32
    pub bonus_wallet: Pubkey,           // 32
    pub nft_rewarder: Pubkey,           // 32 - NFTRewarder program/vault

    /// Global statistics
    pub total_liability: u64,           // 8 - Total potential payouts for pending bets
    pub unsettled_count: u64,           // 8 - Number of unsettled orders
    pub total_bets: u64,                // 8 - Lifetime total bets placed
    pub total_filled: u64,              // 8 - Lifetime filled (won) bets
    pub total_not_filled: u64,          // 8 - Lifetime not-filled (lost) bets
    pub total_volume: u64,              // 8 - Lifetime volume wagered (lamports)
    pub house_profit: i64,              // 8 - Cumulative house P&L (can be negative)

    // Trading accounting
    pub total_payout: u64,              // 8 - Gross SOL paid out to winners
    pub largest_bet: u64,               // 8 - Biggest single bet ever
    pub largest_payout: u64,            // 8 - Biggest single payout ever

    // Fee tracking (from unfilled settlements)
    pub total_referral_paid: u64,       // 8 - Tier1 + Tier2 referral rewards
    pub total_nft_rewards_paid: u64,    // 8 - NFT holder rewards
    pub total_platform_fees_paid: u64,  // 8 - Platform fees
    pub total_bonus_paid: u64,          // 8 - Bonus pool contributions

    // LP activity
    pub total_deposited: u64,           // 8 - Lifetime SOL deposited by LPs
    pub total_withdrawn: u64,           // 8 - Lifetime SOL withdrawn by LPs
    pub lp_deposits_count: u64,         // 8 - Number of LP deposits
    pub lp_withdrawals_count: u64,      // 8 - Number of LP withdrawals

    /// Reserved space for future fields (104 bytes)
    pub _reserved: [u8; 104],           // 104
}

impl ClearingHouseState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // settler
        32 + // vault
        32 + // lp_mint
        32 + // lp_authority
        1 + // vault_bump
        1 + // lp_mint_bump
        1 + // lp_authority_bump
        1 + // paused
        2 + // fate_fee_bps
        2 + // max_bet_bps
        8 + // min_bet
        8 + // bet_timeout
        2 + // referral_bps
        2 + // tier2_referral_bps
        2 + // nft_reward_bps
        2 + // platform_fee_bps
        2 + // bonus_bps
        32 + // platform_wallet
        32 + // bonus_wallet
        32 + // nft_rewarder
        8 + // total_liability
        8 + // unsettled_count
        8 + // total_bets
        8 + // total_filled
        8 + // total_not_filled
        8 + // total_volume
        8 + // house_profit
        8 + // total_payout
        8 + // largest_bet
        8 + // largest_payout
        8 + // total_referral_paid
        8 + // total_nft_rewards_paid
        8 + // total_platform_fees_paid
        8 + // total_bonus_paid
        8 + // total_deposited
        8 + // total_withdrawn
        8 + // lp_deposits_count
        8 + // lp_withdrawals_count
        104; // _reserved

    /// Get net balance available for betting (vault - rent - liability)
    pub fn net_balance(&self, vault_lamports: u64) -> u64 {
        let rent_exempt = Rent::get()
            .unwrap()
            .minimum_balance(8); // 8-byte discriminator for vault PDA

        vault_lamports
            .saturating_sub(rent_exempt)
            .saturating_sub(self.total_liability)
    }
}

impl Default for ClearingHouseState {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(),
            settler: Pubkey::default(),
            vault: Pubkey::default(),
            lp_mint: Pubkey::default(),
            lp_authority: Pubkey::default(),
            vault_bump: 0,
            lp_mint_bump: 0,
            lp_authority_bump: 0,
            paused: false,
            fate_fee_bps: 0,
            max_bet_bps: 0,
            min_bet: 0,
            bet_timeout: 0,
            referral_bps: 0,
            tier2_referral_bps: 0,
            nft_reward_bps: 0,
            platform_fee_bps: 0,
            bonus_bps: 0,
            platform_wallet: Pubkey::default(),
            bonus_wallet: Pubkey::default(),
            nft_rewarder: Pubkey::default(),
            total_liability: 0,
            unsettled_count: 0,
            total_bets: 0,
            total_filled: 0,
            total_not_filled: 0,
            total_volume: 0,
            house_profit: 0,
            total_payout: 0,
            largest_bet: 0,
            largest_payout: 0,
            total_referral_paid: 0,
            total_nft_rewards_paid: 0,
            total_platform_fees_paid: 0,
            total_bonus_paid: 0,
            total_deposited: 0,
            total_withdrawn: 0,
            lp_deposits_count: 0,
            lp_withdrawals_count: 0,
            _reserved: [0; 104],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clearing_house_size() {
        assert_eq!(ClearingHouseState::LEN, 546);
    }
}
