use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct ReclaimExpiredOrder<'info> {
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
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ReclaimExpiredOrder>) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let player_state = &mut ctx.accounts.player_state;
    let fate_order = &ctx.accounts.fate_order;
    let clock = Clock::get()?;

    // Order must be pending
    require!(
        fate_order.status == FateOrderStatus::Pending,
        FateSwapError::OrderNotPending
    );

    // Check timeout has elapsed (strictly greater, not >=)
    let elapsed = clock.unix_timestamp - fate_order.timestamp;
    require!(
        elapsed > clearing_house.bet_timeout,
        FateSwapError::OrderNotExpired
    );

    // Read order fields before any mutable borrows
    let amount = fate_order.amount;
    let potential_payout = fate_order.potential_payout;
    let order_key = fate_order.key();

    // Refund wager from vault to player via CPI (vault is system-owned PDA)
    let vault_bump = clearing_house.vault_bump;
    let vault_seeds = &[b"vault".as_ref(), &[vault_bump]];
    let vault_signer = &[&vault_seeds[..]];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.player.to_account_info(),
            },
            vault_signer,
        ),
        amount,
    )?;

    // Update global state (decrease liability and unsettled count, but NOT total_bets)
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
    emit!(FateOrderReclaimed {
        player: ctx.accounts.player.key(),
        order: order_key,
        refund_amount: amount,
        timestamp: clock.unix_timestamp,
    });

    // FateOrder account is closed via close = player in #[account] macro
    // Rent is returned to player automatically

    Ok(())
}
