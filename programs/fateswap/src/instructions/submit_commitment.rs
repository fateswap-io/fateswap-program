use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
#[instruction(player_key: Pubkey)]
pub struct SubmitCommitment<'info> {
    #[account(
        seeds = [b"clearing_house"],
        bump
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    #[account(
        init_if_needed,
        payer = settler,
        space = PlayerState::LEN,
        seeds = [b"player_state", player_key.as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    /// CHECK: Player wallet (can be any pubkey)
    pub player: AccountInfo<'info>,

    #[account(mut)]
    pub settler: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SubmitCommitment>,
    _player_key: Pubkey,
    commitment_hash: [u8; 32],
    nonce: u64,
) -> Result<()> {
    let clearing_house = &ctx.accounts.clearing_house;
    let player_state = &mut ctx.accounts.player_state;
    let clock = Clock::get()?;

    // Only settler can submit commitments
    require!(
        ctx.accounts.settler.key() == clearing_house.settler,
        FateSwapError::UnauthorizedSettler
    );

    // Check not paused
    require!(!clearing_house.paused, FateSwapError::Paused);

    // If this is first initialization, set the player and bump
    if player_state.player == Pubkey::default() {
        player_state.player = ctx.accounts.player.key();
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

    // Store pending commitment
    player_state.pending_commitment = commitment_hash;
    player_state.pending_nonce = nonce;

    // Emit event
    emit!(CommitmentSubmitted {
        player: ctx.accounts.player.key(),
        commitment_hash,
        nonce,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
