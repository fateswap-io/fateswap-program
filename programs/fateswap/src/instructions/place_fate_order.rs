use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::*;
use crate::events::*;
use crate::math::*;

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct PlaceFateOrder<'info> {
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
        seeds = [b"player_state", player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        init,
        payer = player,
        space = FateOrder::LEN,
        seeds = [b"fate_order", player.key().as_ref(), nonce.to_le_bytes().as_ref()],
        bump
    )]
    pub fate_order: Account<'info, FateOrder>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<PlaceFateOrder>,
    nonce: u64,
    amount: u64,
    multiplier_bps: u32,
    token_mint: Pubkey,
    token_amount: u64,
) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let player_state = &mut ctx.accounts.player_state;
    let fate_order = &mut ctx.accounts.fate_order;
    let clock = Clock::get()?;

    // Check not paused
    require!(!clearing_house.paused, FateSwapError::Paused);

    // Validate multiplier
    require!(
        is_valid_multiplier(multiplier_bps),
        FateSwapError::InvalidMultiplier
    );

    // Check nonce matches pending nonce
    require!(
        nonce == player_state.pending_nonce,
        FateSwapError::NonceMismatch
    );

    // Check commitment exists (not all zeros)
    require!(
        player_state.pending_commitment != [0; 32],
        FateSwapError::NoCommitment
    );

    // Check no active order
    require!(
        !player_state.has_active_order,
        FateSwapError::ActiveOrderExists
    );

    // Validate bet size
    require!(amount >= clearing_house.min_bet, FateSwapError::BetTooSmall);

    // Calculate potential payout
    let payout_u128 = (amount as u128)
        .checked_mul(multiplier_bps as u128)
        .ok_or(FateSwapError::MathOverflow)?
        .checked_div(MULTIPLIER_BASE as u128)
        .ok_or(FateSwapError::MathOverflow)?;

    let potential_payout = u64::try_from(payout_u128)
        .map_err(|_| FateSwapError::MathOverflow)?;

    // Calculate max bet for this multiplier
    let vault_balance = ctx.accounts.vault.lamports();
    let net_balance = clearing_house.net_balance(vault_balance);
    let base_max = net_balance
        .checked_mul(clearing_house.max_bet_bps as u64)
        .ok_or(FateSwapError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FateSwapError::MathOverflow)?;

    let max_bet = calculate_max_bet(base_max, multiplier_bps)?;

    require!(amount <= max_bet, FateSwapError::BetTooLarge);

    // Check vault can cover potential payout
    require!(
        net_balance >= potential_payout,
        FateSwapError::InsufficientVaultBalance
    );

    // Transfer SOL to vault
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    // Initialize FateOrder
    fate_order.player = ctx.accounts.player.key();
    fate_order.amount = amount;
    fate_order.multiplier_bps = multiplier_bps;
    fate_order.potential_payout = potential_payout;
    fate_order.commitment_hash = player_state.pending_commitment;
    fate_order.nonce = nonce;
    fate_order.status = FateOrderStatus::Pending;
    fate_order.timestamp = clock.unix_timestamp;
    fate_order.token_mint = token_mint;
    fate_order.token_amount = token_amount;
    fate_order.bump = ctx.bumps.fate_order;
    fate_order._reserved = [0; 45];

    // Update global state
    clearing_house.total_liability = clearing_house
        .total_liability
        .checked_add(potential_payout)
        .ok_or(FateSwapError::MathOverflow)?;
    clearing_house.unsettled_count = clearing_house
        .unsettled_count
        .checked_add(1)
        .ok_or(FateSwapError::MathOverflow)?;

    // Track largest bet
    if amount > clearing_house.largest_bet {
        clearing_house.largest_bet = amount;
    }

    // Update player state
    player_state.has_active_order = true;
    player_state.active_order = fate_order.key();
    player_state.nonce = player_state.nonce.checked_add(1).ok_or(FateSwapError::MathOverflow)?;
    player_state.pending_commitment = [0; 32]; // Clear pending commitment

    // Emit event
    emit!(FateOrderPlaced {
        player: ctx.accounts.player.key(),
        order: fate_order.key(),
        amount,
        multiplier_bps,
        potential_payout,
        commitment_hash: fate_order.commitment_hash,
        nonce,
        token_mint,
        token_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
