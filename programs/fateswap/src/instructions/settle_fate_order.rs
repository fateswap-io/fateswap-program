use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program::hash::hashv;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct SettleFateOrder<'info> {
    #[account(
        mut,
        seeds = [b"clearing_house"],
        bump
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = clearing_house.vault_bump
    )]
    /// CHECK: Vault PDA that holds SOL
    pub vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"player_state", fate_order.player.as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        mut,
        close = player,
        has_one = player,
        seeds = [b"fate_order", fate_order.player.as_ref(), fate_order.nonce.to_le_bytes().as_ref()],
        bump = fate_order.bump
    )]
    pub fate_order: Account<'info, FateOrder>,

    #[account(mut)]
    /// CHECK: Player wallet (validated via fate_order.player)
    pub player: AccountInfo<'info>,

    // Tier-1 referrer (optional)
    #[account(mut)]
    /// CHECK: Tier-1 referrer wallet
    pub tier1_referrer: Option<AccountInfo<'info>>,

    #[account(mut)]
    /// Tier-1 referrer's ReferralState (optional)
    pub tier1_referral_state: Option<Account<'info, ReferralState>>,

    // Tier-2 referrer (optional)
    #[account(mut)]
    /// CHECK: Tier-2 referrer wallet
    pub tier2_referrer: Option<AccountInfo<'info>>,

    #[account(mut)]
    /// Tier-2 referrer's ReferralState (optional)
    pub tier2_referral_state: Option<Account<'info, ReferralState>>,

    // 5-way split wallets — validated against ClearingHouseState addresses
    #[account(
        mut,
        constraint = nft_rewarder.key() == clearing_house.nft_rewarder @ FateSwapError::InvalidNFTRewarder
    )]
    /// CHECK: NFT rewarder program/vault (validated against clearing_house.nft_rewarder)
    pub nft_rewarder: AccountInfo<'info>,

    #[account(
        mut,
        constraint = platform_wallet.key() == clearing_house.platform_wallet @ FateSwapError::InvalidPlatformWallet
    )]
    /// CHECK: Platform wallet (validated against clearing_house.platform_wallet)
    pub platform_wallet: AccountInfo<'info>,

    #[account(
        mut,
        constraint = bonus_wallet.key() == clearing_house.bonus_wallet @ FateSwapError::InvalidBonusWallet
    )]
    /// CHECK: Bonus wallet (validated against clearing_house.bonus_wallet)
    pub bonus_wallet: AccountInfo<'info>,

    #[account(mut)]
    pub settler: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SettleFateOrder>,
    filled: bool,
    server_seed: [u8; 32],
) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let player_state = &mut ctx.accounts.player_state;
    let fate_order = &ctx.accounts.fate_order;
    let clock = Clock::get()?;

    // Only settler can settle
    require!(
        ctx.accounts.settler.key() == clearing_house.settler,
        FateSwapError::UnauthorizedSettler
    );

    // Order must be pending
    require!(
        fate_order.status == FateOrderStatus::Pending,
        FateSwapError::OrderNotPending
    );

    // Verify commitment (SHA256 check)
    let computed_hash = hashv(&[&server_seed]).to_bytes();
    require!(
        computed_hash == fate_order.commitment_hash,
        FateSwapError::InvalidServerSeed
    );

    // Reject expired orders — they can only be reclaimed, not settled
    let elapsed = clock.unix_timestamp
        .checked_sub(fate_order.timestamp)
        .ok_or(FateSwapError::MathOverflow)?;
    require!(
        elapsed <= clearing_house.bet_timeout,
        FateSwapError::OrderExpired
    );

    // Validate referrer accounts match player_state
    if let Some(ref tier1_ref) = ctx.accounts.tier1_referrer {
        if player_state.referrer != Pubkey::default() {
            require!(
                tier1_ref.key() == player_state.referrer,
                FateSwapError::InvalidReferrer
            );
        }
    }

    if let Some(ref tier2_ref) = ctx.accounts.tier2_referrer {
        if player_state.tier2_referrer != Pubkey::default() {
            require!(
                tier2_ref.key() == player_state.tier2_referrer,
                FateSwapError::InvalidReferrer
            );
        }
    }

    // Read order fields into local vars before any mutable borrows (borrow checker)
    let amount = fate_order.amount;
    let multiplier_bps = fate_order.multiplier_bps;
    let potential_payout = fate_order.potential_payout;
    let order_key = fate_order.key();

    // Compute vault signer seeds for CPI transfers
    let vault_bump = clearing_house.vault_bump;
    let vault_seeds: &[&[u8]] = &[b"vault".as_ref(), &[vault_bump]];
    let vault_signer = &[vault_seeds];

    // Process settlement
    if filled {
        // Player wins - transfer payout from vault to player via CPI
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.player.to_account_info(),
                },
                vault_signer,
            ),
            potential_payout,
        )?;

        // Update global stats
        clearing_house.total_bets = clearing_house
            .total_bets
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.total_filled = clearing_house
            .total_filled
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.total_volume = clearing_house
            .total_volume
            .checked_add(amount)
            .ok_or(FateSwapError::MathOverflow)?;

        let profit_change = (potential_payout as i64)
            .checked_sub(amount as i64)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.house_profit = clearing_house
            .house_profit
            .checked_sub(profit_change)
            .ok_or(FateSwapError::MathOverflow)?;

        // Track payout accounting
        clearing_house.total_payout = clearing_house
            .total_payout
            .checked_add(potential_payout)
            .ok_or(FateSwapError::MathOverflow)?;
        if potential_payout > clearing_house.largest_payout {
            clearing_house.largest_payout = potential_payout;
        }

        // Update player stats
        player_state.total_orders = player_state
            .total_orders
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        player_state.total_wagered = player_state
            .total_wagered
            .checked_add(amount)
            .ok_or(FateSwapError::MathOverflow)?;

        let player_profit = potential_payout
            .checked_sub(amount)
            .ok_or(FateSwapError::MathOverflow)?;
        player_state.total_won = player_state
            .total_won
            .checked_add(player_profit)
            .ok_or(FateSwapError::MathOverflow)?;
        player_state.net_pnl = player_state
            .net_pnl
            .checked_add(player_profit as i64)
            .ok_or(FateSwapError::MathOverflow)?;
    } else {
        // Player loses - execute 5-way revenue split

        // 1. Tier-1 referral reward
        if clearing_house.referral_bps > 0 {
            if let (Some(ref mut tier1_ref), Some(ref mut tier1_rs)) = (
                ctx.accounts.tier1_referrer.as_mut(),
                ctx.accounts.tier1_referral_state.as_mut(),
            ) {
                let reward = amount
                    .checked_mul(clearing_house.referral_bps as u64)
                    .ok_or(FateSwapError::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(FateSwapError::MathOverflow)?;

                if reward > 0 {
                    // Referral transfers are best-effort — log failures but don't block settlement
                    if send_reward(
                        &ctx.accounts.vault,
                        tier1_ref,
                        &ctx.accounts.system_program,
                        vault_signer,
                        reward,
                    ).is_ok() {
                        tier1_rs.total_earnings = tier1_rs.total_earnings.saturating_add(reward);
                        clearing_house.total_referral_paid = clearing_house
                            .total_referral_paid
                            .saturating_add(reward);

                        emit!(RewardPaid {
                            reward_type: 0, // tier1_referral
                            recipient: tier1_ref.key(),
                            amount: reward,
                            order: order_key,
                            timestamp: clock.unix_timestamp,
                        });
                    } else {
                        emit!(RewardFailed {
                            reward_type: 0,
                            recipient: tier1_ref.key(),
                            amount: reward,
                            order: order_key,
                            timestamp: clock.unix_timestamp,
                        });
                    }
                }
            }
        }

        // 2. Tier-2 referral reward
        if clearing_house.tier2_referral_bps > 0 {
            if let (Some(ref mut tier2_ref), Some(ref mut tier2_rs)) = (
                ctx.accounts.tier2_referrer.as_mut(),
                ctx.accounts.tier2_referral_state.as_mut(),
            ) {
                let reward = amount
                    .checked_mul(clearing_house.tier2_referral_bps as u64)
                    .ok_or(FateSwapError::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(FateSwapError::MathOverflow)?;

                if reward > 0 {
                    if send_reward(
                        &ctx.accounts.vault,
                        tier2_ref,
                        &ctx.accounts.system_program,
                        vault_signer,
                        reward,
                    ).is_ok() {
                        tier2_rs.total_earnings = tier2_rs.total_earnings.saturating_add(reward);
                        clearing_house.total_referral_paid = clearing_house
                            .total_referral_paid
                            .saturating_add(reward);

                        emit!(RewardPaid {
                            reward_type: 1, // tier2_referral
                            recipient: tier2_ref.key(),
                            amount: reward,
                            order: order_key,
                            timestamp: clock.unix_timestamp,
                        });
                    } else {
                        emit!(RewardFailed {
                            reward_type: 1,
                            recipient: tier2_ref.key(),
                            amount: reward,
                            order: order_key,
                            timestamp: clock.unix_timestamp,
                        });
                    }
                }
            }
        }

        // 3. NFT reward — must succeed (wallet validated against clearing_house)
        if clearing_house.nft_reward_bps > 0 {
            let reward = amount
                .checked_mul(clearing_house.nft_reward_bps as u64)
                .ok_or(FateSwapError::MathOverflow)?
                .checked_div(10000)
                .ok_or(FateSwapError::MathOverflow)?;

            if reward > 0 {
                send_reward(
                    &ctx.accounts.vault,
                    &ctx.accounts.nft_rewarder,
                    &ctx.accounts.system_program,
                    vault_signer,
                    reward,
                )?;
                clearing_house.total_nft_rewards_paid = clearing_house
                    .total_nft_rewards_paid
                    .checked_add(reward)
                    .ok_or(FateSwapError::MathOverflow)?;

                emit!(RewardPaid {
                    reward_type: 2, // nft_reward
                    recipient: ctx.accounts.nft_rewarder.key(),
                    amount: reward,
                    order: order_key,
                    timestamp: clock.unix_timestamp,
                });
            }
        }

        // 4. Platform fee — must succeed
        if clearing_house.platform_fee_bps > 0 {
            let reward = amount
                .checked_mul(clearing_house.platform_fee_bps as u64)
                .ok_or(FateSwapError::MathOverflow)?
                .checked_div(10000)
                .ok_or(FateSwapError::MathOverflow)?;

            if reward > 0 {
                send_reward(
                    &ctx.accounts.vault,
                    &ctx.accounts.platform_wallet,
                    &ctx.accounts.system_program,
                    vault_signer,
                    reward,
                )?;
                clearing_house.total_platform_fees_paid = clearing_house
                    .total_platform_fees_paid
                    .checked_add(reward)
                    .ok_or(FateSwapError::MathOverflow)?;

                emit!(RewardPaid {
                    reward_type: 3, // platform_fee
                    recipient: ctx.accounts.platform_wallet.key(),
                    amount: reward,
                    order: order_key,
                    timestamp: clock.unix_timestamp,
                });
            }
        }

        // 5. Bonus — must succeed
        if clearing_house.bonus_bps > 0 {
            let reward = amount
                .checked_mul(clearing_house.bonus_bps as u64)
                .ok_or(FateSwapError::MathOverflow)?
                .checked_div(10000)
                .ok_or(FateSwapError::MathOverflow)?;

            if reward > 0 {
                send_reward(
                    &ctx.accounts.vault,
                    &ctx.accounts.bonus_wallet,
                    &ctx.accounts.system_program,
                    vault_signer,
                    reward,
                )?;
                clearing_house.total_bonus_paid = clearing_house
                    .total_bonus_paid
                    .checked_add(reward)
                    .ok_or(FateSwapError::MathOverflow)?;

                emit!(RewardPaid {
                    reward_type: 4, // bonus
                    recipient: ctx.accounts.bonus_wallet.key(),
                    amount: reward,
                    order: order_key,
                    timestamp: clock.unix_timestamp,
                });
            }
        }

        // Update global stats
        clearing_house.total_bets = clearing_house
            .total_bets
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.total_not_filled = clearing_house
            .total_not_filled
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.total_volume = clearing_house
            .total_volume
            .checked_add(amount)
            .ok_or(FateSwapError::MathOverflow)?;
        clearing_house.house_profit = clearing_house
            .house_profit
            .checked_add(amount as i64)
            .ok_or(FateSwapError::MathOverflow)?;

        // Update player stats
        player_state.total_orders = player_state
            .total_orders
            .checked_add(1)
            .ok_or(FateSwapError::MathOverflow)?;
        player_state.total_wagered = player_state
            .total_wagered
            .checked_add(amount)
            .ok_or(FateSwapError::MathOverflow)?;
        player_state.net_pnl = player_state
            .net_pnl
            .checked_sub(amount as i64)
            .ok_or(FateSwapError::MathOverflow)?;
    }

    // Update liability and unsettled count
    clearing_house.total_liability = clearing_house
        .total_liability
        .checked_sub(potential_payout)
        .ok_or(FateSwapError::MathOverflow)?;
    clearing_house.unsettled_count = clearing_house
        .unsettled_count
        .checked_sub(1)
        .ok_or(FateSwapError::MathOverflow)?;

    // Clear player's active order
    player_state.has_active_order = false;
    player_state.active_order = Pubkey::default();

    // Emit event
    emit!(FateOrderSettled {
        player: ctx.accounts.player.key(),
        order: order_key,
        filled,
        amount,
        multiplier_bps,
        payout: if filled { potential_payout } else { 0 },
        server_seed,
        commitment_hash: fate_order.commitment_hash,
        nonce: fate_order.nonce,
        timestamp: clock.unix_timestamp,
    });

    // FateOrder account is closed via close = player in #[account] macro
    // Rent is returned to player automatically

    Ok(())
}

/// Helper function to send rewards via system_program::transfer CPI (non-blocking)
/// Returns Ok(()) if successful, Err otherwise (errors are ignored by caller)
fn send_reward<'info>(
    vault: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    vault_signer: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    system_program::transfer(
        CpiContext::new_with_signer(
            system_program.to_account_info(),
            system_program::Transfer {
                from: vault.to_account_info(),
                to: recipient.to_account_info(),
            },
            vault_signer,
        ),
        amount,
    )
}
