use anchor_lang::prelude::*;

#[error_code]
pub enum FateSwapError {
    #[msg("Protocol is paused")]
    Paused,                         // 6000

    #[msg("Amount cannot be zero")]
    ZeroAmount,                     // 6001

    #[msg("Deposit too small")]
    DepositTooSmall,                // 6002

    #[msg("Withdrawal too small")]
    WithdrawTooSmall,               // 6003

    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,          // 6004

    #[msg("Math overflow")]
    MathOverflow,                   // 6005

    #[msg("Invalid configuration")]
    InvalidConfig,                  // 6006

    #[msg("Invalid multiplier")]
    InvalidMultiplier,              // 6007

    #[msg("Nonce mismatch")]
    NonceMismatch,                  // 6008

    #[msg("No commitment")]
    NoCommitment,                   // 6009

    #[msg("Active order exists")]
    ActiveOrderExists,              // 6010

    #[msg("Bet too small")]
    BetTooSmall,                    // 6011

    #[msg("Bet too large")]
    BetTooLarge,                    // 6012

    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,       // 6013

    #[msg("Order not pending")]
    OrderNotPending,                // 6014

    #[msg("Order expired")]
    OrderExpired,                   // 6015

    #[msg("Order not expired")]
    OrderNotExpired,                // 6016

    #[msg("Unauthorized settler")]
    UnauthorizedSettler,            // 6017

    #[msg("Invalid player")]
    InvalidPlayer,                  // 6018

    #[msg("Invalid server seed")]
    InvalidServerSeed,              // 6019

    #[msg("Invalid referrer")]
    InvalidReferrer,                // 6020

    #[msg("Self referral not allowed")]
    SelfReferral,                   // 6021

    #[msg("Referrer already set")]
    ReferrerAlreadySet,             // 6022

    #[msg("Invalid NFT rewarder")]
    InvalidNFTRewarder,             // 6023

    #[msg("Invalid platform wallet")]
    InvalidPlatformWallet,          // 6024

    #[msg("Invalid bonus wallet")]
    InvalidBonusWallet,             // 6025
}
