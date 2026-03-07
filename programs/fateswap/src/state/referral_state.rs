use anchor_lang::prelude::*;

/// ReferralState account (~80 bytes)
/// Tracks referrer statistics
#[account]
#[derive(Default)]
pub struct ReferralState {
    /// The referrer wallet
    pub referrer: Pubkey,               // 32

    /// Total number of referrals (players who set this wallet as referrer)
    pub total_referrals: u64,           // 8

    /// Total earnings from referral rewards (cumulative)
    pub total_earnings: u64,            // 8

    /// PDA bump
    pub bump: u8,                       // 1

    /// Reserved space for future fields
    pub _reserved: [u8; 23],            // 23
}

impl ReferralState {
    pub const LEN: usize = 8 + // discriminator
        32 + // referrer
        8 + // total_referrals
        8 + // total_earnings
        1 + // bump
        23; // _reserved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referral_state_size() {
        assert_eq!(ReferralState::LEN, 80); // 8 + 72
    }
}
