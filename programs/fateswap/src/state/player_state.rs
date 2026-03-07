use anchor_lang::prelude::*;

/// PlayerState account (249 bytes including 8-byte discriminator)
/// Persistent per-player state including commitment and stats
#[account]
#[derive(Default)]
pub struct PlayerState {
    /// Player wallet
    pub player: Pubkey,                 // 32

    /// Pending commitment hash (SHA256 of server seed)
    pub pending_commitment: [u8; 32],   // 32

    /// Pending nonce
    pub pending_nonce: u64,             // 8

    /// Next nonce to use
    pub nonce: u64,                     // 8

    /// Has active order flag
    pub has_active_order: bool,         // 1

    /// Active order PDA
    pub active_order: Pubkey,           // 32

    /// Tier-1 referrer (one-time set)
    pub referrer: Pubkey,               // 32

    /// Tier-2 referrer (auto-resolved from referrer's referrer at set_referrer time)
    pub tier2_referrer: Pubkey,         // 32

    /// Player statistics
    pub total_orders: u64,              // 8
    pub total_wagered: u64,             // 8
    pub total_won: u64,                 // 8 - profit only, not total payouts
    pub net_pnl: i64,                   // 8 - can be negative

    /// PDA bump
    pub bump: u8,                       // 1

    /// Reserved space for future fields
    pub _reserved: [u8; 31],            // 31
}

impl PlayerState {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        32 + // pending_commitment
        8 + // pending_nonce
        8 + // nonce
        1 + // has_active_order
        32 + // active_order
        32 + // referrer
        32 + // tier2_referrer
        8 + // total_orders
        8 + // total_wagered
        8 + // total_won
        8 + // net_pnl
        1 + // bump
        31; // _reserved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_state_size() {
        assert_eq!(PlayerState::LEN, 249); // 8 + 241
    }
}
