use anchor_lang::prelude::*;

/// Status of a FateOrder
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum FateOrderStatus {
    Pending,
    Filled,
    NotFilled,
    Expired,
}

impl Default for FateOrderStatus {
    fn default() -> Self {
        FateOrderStatus::Pending
    }
}

/// FateOrder state account (195 bytes including 8-byte discriminator)
/// Represents a single prediction market order
#[account]
pub struct FateOrder {
    /// Player who placed the order
    pub player: Pubkey,                 // 32

    /// Wager amount in lamports
    pub amount: u64,                    // 8

    /// Multiplier in basis points (101000 = 1.01x, 1000000 = 10.0x)
    pub multiplier_bps: u32,            // 4

    /// Potential payout if filled (amount * multiplier_bps / 100000)
    pub potential_payout: u64,          // 8

    /// Commitment hash (SHA256 of server seed)
    pub commitment_hash: [u8; 32],      // 32

    /// Nonce for this order
    pub nonce: u64,                     // 8

    /// Order status
    pub status: FateOrderStatus,        // 1

    /// Timestamp when order was placed
    pub timestamp: i64,                 // 8

    /// Token mint (metadata only, SPL tokens not handled on-chain)
    pub token_mint: Pubkey,             // 32

    /// Token amount (metadata only)
    pub token_amount: u64,              // 8

    /// PDA bump
    pub bump: u8,                       // 1

    /// Reserved space for future fields
    pub _reserved: [u8; 45],            // 45
}

impl FateOrder {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // amount
        4 + // multiplier_bps
        8 + // potential_payout
        32 + // commitment_hash
        8 + // nonce
        1 + // status
        8 + // timestamp
        32 + // token_mint
        8 + // token_amount
        1 + // bump
        45; // _reserved
}

impl Default for FateOrder {
    fn default() -> Self {
        Self {
            player: Pubkey::default(),
            amount: 0,
            multiplier_bps: 0,
            potential_payout: 0,
            commitment_hash: [0; 32],
            nonce: 0,
            status: FateOrderStatus::default(),
            timestamp: 0,
            token_mint: Pubkey::default(),
            token_amount: 0,
            bump: 0,
            _reserved: [0; 45],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fate_order_size() {
        assert_eq!(FateOrder::LEN, 195); // 8 + 187
    }
}
