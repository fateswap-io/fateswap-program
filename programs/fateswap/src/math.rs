use anchor_lang::prelude::*;
use crate::errors::FateSwapError;

/// Constants for multiplier system
pub const MIN_MULTIPLIER: u32 = 101_000;     // 1.01x
pub const MAX_MULTIPLIER: u32 = 1_000_000;   // 10.0x
pub const MULTIPLIER_BASE: u32 = 100_000;    // 1.0x = 100,000

/// MINIMUM_LIQUIDITY burned on first deposit (donation attack prevention)
pub const MINIMUM_LIQUIDITY: u64 = 10_000;  // 10,000 lamports

/// Calculate maximum bet for a given multiplier
/// Inverse scaling: max_bet = base_max * 200000 / multiplier_bps
/// This ensures that max_bet * (multiplier - 1) is roughly constant
pub fn calculate_max_bet(base_max: u64, multiplier_bps: u32) -> Result<u64> {
    // Reference point: 2x (200000 bps)
    const REFERENCE_MULTIPLIER: u32 = 200_000;

    let base_u128 = base_max as u128;
    let ref_u128 = REFERENCE_MULTIPLIER as u128;
    let mult_u128 = multiplier_bps as u128;

    let max_bet_u128 = base_u128
        .checked_mul(ref_u128)
        .ok_or(FateSwapError::MathOverflow)?
        .checked_div(mult_u128)
        .ok_or(FateSwapError::MathOverflow)?;

    u64::try_from(max_bet_u128).map_err(|_| FateSwapError::MathOverflow.into())
}

/// Validate multiplier is one of the 210 discrete allowed values
/// Tier 1 (1.01x - 1.99x): 0.01x steps = 99 values
/// Tier 2 (2.00x - 2.98x): 0.02x steps = 50 values
/// Tier 3 (3.00x - 3.95x): 0.05x steps = 20 values
/// Tier 4 (4.0x - 5.9x):   0.1x steps  = 20 values
/// Tier 5 (6.0x - 9.8x):   0.2x steps  = 20 values
/// Tier 6 (10.0x):         single value = 1 value
/// Total: 210 discrete values
#[inline(always)]
pub fn is_valid_multiplier(multiplier_bps: u32) -> bool {
    if multiplier_bps < MIN_MULTIPLIER || multiplier_bps > MAX_MULTIPLIER {
        return false;
    }

    // Tier 1: 1.01x to 1.99x (101000 to 199000), steps of 1000 (0.01x)
    if multiplier_bps >= 101_000 && multiplier_bps < 200_000 {
        return (multiplier_bps - 101_000) % 1_000 == 0;
    }

    // Tier 2: 2.00x to 2.98x (200000 to 298000), steps of 2000 (0.02x)
    if multiplier_bps >= 200_000 && multiplier_bps < 300_000 {
        return (multiplier_bps - 200_000) % 2_000 == 0;
    }

    // Tier 3: 3.00x to 3.95x (300000 to 395000), steps of 5000 (0.05x)
    if multiplier_bps >= 300_000 && multiplier_bps < 400_000 {
        return (multiplier_bps - 300_000) % 5_000 == 0;
    }

    // Tier 4: 4.0x to 5.9x (400000 to 590000), steps of 10000 (0.1x)
    if multiplier_bps >= 400_000 && multiplier_bps < 600_000 {
        return (multiplier_bps - 400_000) % 10_000 == 0;
    }

    // Tier 5: 6.0x to 9.8x (600000 to 980000), steps of 20000 (0.2x)
    if multiplier_bps >= 600_000 && multiplier_bps < 1_000_000 {
        return (multiplier_bps - 600_000) % 20_000 == 0;
    }

    // Tier 6: exactly 10.0x (1000000)
    multiplier_bps == 1_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_bet_at_2x() {
        let base = 1_000_000;
        let max = calculate_max_bet(base, 200_000).unwrap();
        assert_eq!(max, base);
    }

    #[test]
    fn test_multiplier_validation_tier1() {
        assert!(is_valid_multiplier(101_000)); // 1.01x
        assert!(is_valid_multiplier(150_000)); // 1.50x
        assert!(is_valid_multiplier(199_000)); // 1.99x
        assert!(!is_valid_multiplier(101_500)); // invalid step
    }

    #[test]
    fn test_multiplier_validation_tier2() {
        assert!(is_valid_multiplier(200_000)); // 2.00x
        assert!(is_valid_multiplier(250_000)); // 2.50x
        assert!(!is_valid_multiplier(201_000)); // invalid step
    }

    #[test]
    fn test_multiplier_validation_tier3() {
        assert!(is_valid_multiplier(300_000)); // 3.00x
        assert!(is_valid_multiplier(395_000)); // 3.95x
        assert!(!is_valid_multiplier(301_000)); // invalid step
    }

    #[test]
    fn test_multiplier_validation_tier4() {
        assert!(is_valid_multiplier(400_000)); // 4.0x
        assert!(is_valid_multiplier(500_000)); // 5.0x
        assert!(!is_valid_multiplier(405_000)); // invalid step
    }

    #[test]
    fn test_multiplier_validation_tier5() {
        assert!(is_valid_multiplier(600_000)); // 6.0x
        assert!(is_valid_multiplier(980_000)); // 9.8x
        assert!(!is_valid_multiplier(610_000)); // invalid step
    }

    #[test]
    fn test_multiplier_validation_tier6() {
        assert!(is_valid_multiplier(1_000_000)); // 10.0x
    }

    #[test]
    fn test_multiplier_out_of_range() {
        assert!(!is_valid_multiplier(100_000)); // 1.0x, below min
        assert!(!is_valid_multiplier(1_020_000)); // 10.2x, above max
    }
}
