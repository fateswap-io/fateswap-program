use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

/// Admin variant of set_referrer — settler is signer/payer instead of player.
/// Follows the submit_commitment pattern: settler signs, player is CHECK, PDAs use init_if_needed.
#[derive(Accounts)]
#[instruction(player_key: Pubkey, referrer_key: Pubkey)]
pub struct AdminSetReferrer<'info> {
    #[account(
        seeds = [b"clearing_house"],
        bump
    )]
    pub clearing_house: Box<Account<'info, ClearingHouseState>>,

    #[account(
        init_if_needed,
        payer = settler,
        space = PlayerState::LEN,
        seeds = [b"player_state", player_key.as_ref()],
        bump
    )]
    pub player_state: Box<Account<'info, PlayerState>>,

    #[account(
        init_if_needed,
        payer = settler,
        space = ReferralState::LEN,
        seeds = [b"referral_state", referrer_key.as_ref()],
        bump
    )]
    pub referral_state: Box<Account<'info, ReferralState>>,

    /// The referrer's PlayerState (optional — only exists if referrer has played before)
    /// Used for tier-2 referrer resolution
    #[account(
        seeds = [b"player_state", referrer.key().as_ref()],
        bump = referrer_player_state.bump
    )]
    pub referrer_player_state: Option<Box<Account<'info, PlayerState>>>,

    /// Settler is the signer and payer for account creation
    #[account(mut)]
    pub settler: Signer<'info>,

    /// CHECK: Player wallet (any pubkey)
    pub player: AccountInfo<'info>,

    /// CHECK: Referrer wallet (any pubkey)
    pub referrer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AdminSetReferrer>,
    player_key: Pubkey,
    referrer_key: Pubkey,
) -> Result<()> {
    let clearing_house = &ctx.accounts.clearing_house;
    let player_state = &mut ctx.accounts.player_state;
    let referral_state = &mut ctx.accounts.referral_state;
    let clock = Clock::get()?;

    // Only settler can call this instruction
    require!(
        ctx.accounts.settler.key() == clearing_house.settler,
        FateSwapError::UnauthorizedSettler
    );

    // Block self-referral
    require!(
        player_key != referrer_key,
        FateSwapError::SelfReferral
    );

    // If PlayerState is newly created (init_if_needed), initialize all fields
    // Same initialization as submit_commitment handler (lines 53-65)
    if player_state.player == Pubkey::default() {
        player_state.player = player_key;
        player_state.bump = ctx.bumps.player_state;
        player_state.nonce = 0;
        player_state.has_active_order = false;
        player_state.active_order = Pubkey::default();
        player_state.referrer = Pubkey::default();
        player_state.tier2_referrer = Pubkey::default();
        player_state.total_orders = 0;
        player_state.total_wagered = 0;
        player_state.total_won = 0;
        player_state.net_pnl = 0;
        player_state._reserved = [0; 31];
    }

    // Can only set referrer once
    require!(
        player_state.referrer == Pubkey::default(),
        FateSwapError::ReferrerAlreadySet
    );

    // Initialize ReferralState if this is first time
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
        if referrer_ps.referrer != Pubkey::default() {
            referrer_ps.referrer
        } else {
            Pubkey::default()
        }
    } else {
        Pubkey::default()
    };

    player_state.tier2_referrer = tier2_referrer;

    // Emit event
    emit!(ReferrerSet {
        player: player_key,
        referrer: referrer_key,
        tier2_referrer,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
