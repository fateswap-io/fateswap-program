use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
#[instruction(referrer_key: Pubkey)]
pub struct SetReferrer<'info> {
    #[account(
        mut,
        seeds = [b"player_state", player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        init_if_needed,
        payer = player,
        space = ReferralState::LEN,
        seeds = [b"referral_state", referrer_key.as_ref()],
        bump
    )]
    pub referral_state: Account<'info, ReferralState>,

    /// The referrer's PlayerState (optional - only exists if referrer has played before)
    /// Validated via PDA seed constraint when present
    #[account(
        seeds = [b"player_state", referrer.key().as_ref()],
        bump = referrer_player_state.bump
    )]
    pub referrer_player_state: Option<Account<'info, PlayerState>>,

    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: Referrer wallet (can be any pubkey)
    pub referrer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SetReferrer>, referrer_key: Pubkey) -> Result<()> {
    let player_state = &mut ctx.accounts.player_state;
    let referral_state = &mut ctx.accounts.referral_state;
    let clock = Clock::get()?;

    // Block self-referral
    require!(
        ctx.accounts.player.key() != referrer_key,
        FateSwapError::SelfReferral
    );

    // Can only set referrer once
    require!(
        player_state.referrer == Pubkey::default(),
        FateSwapError::ReferrerAlreadySet
    );

    // Initialize referral_state if this is first time
    if referral_state.referrer == Pubkey::default() {
        referral_state.referrer = referrer_key;
        referral_state.bump = ctx.bumps.referral_state;
        referral_state.total_referrals = 0;
        referral_state.total_earnings = 0;
        referral_state._reserved = [0; 23];
    }

    // Increment referral count
    referral_state.total_referrals = referral_state
        .total_referrals
        .checked_add(1)
        .ok_or(FateSwapError::MathOverflow)?;

    // Set tier-1 referrer
    player_state.referrer = referrer_key;

    // Auto-resolve tier-2 referrer from referrer's PlayerState
    let tier2_referrer = if let Some(referrer_ps) = &ctx.accounts.referrer_player_state {
        // Referrer has a PlayerState and has a referrer set
        if referrer_ps.referrer != Pubkey::default() {
            referrer_ps.referrer
        } else {
            Pubkey::default()
        }
    } else {
        // Referrer doesn't have a PlayerState yet (hasn't played)
        Pubkey::default()
    };

    // Block circular referral (tier-2 self-referral)
    let tier2_referrer = if tier2_referrer == ctx.accounts.player.key() {
        Pubkey::default()
    } else {
        tier2_referrer
    };

    player_state.tier2_referrer = tier2_referrer;

    // Emit event
    emit!(ReferrerSet {
        player: ctx.accounts.player.key(),
        referrer: referrer_key,
        tier2_referrer,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
