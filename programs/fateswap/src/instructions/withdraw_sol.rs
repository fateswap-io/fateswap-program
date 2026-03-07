use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::state::*;
use crate::errors::*;
use crate::events::*;
use crate::math::*;

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
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
        seeds = [b"lp_mint"],
        bump = clearing_house.lp_mint_bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = withdrawer,
    )]
    pub withdrawer_lp_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawSol>, lp_amount: u64) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    // Check not paused
    require!(!clearing_house.paused, FateSwapError::Paused);

    // Check amount not zero
    require!(lp_amount > 0, FateSwapError::ZeroAmount);

    // Get current vault balance and LP supply
    let vault_balance = ctx.accounts.vault.lamports();
    let lp_supply = ctx.accounts.lp_mint.supply;

    // Calculate available liquidity (vault - rent - liability)
    let rent_exempt = Rent::get()?.minimum_balance(8); // 8-byte discriminator
    let available_liquidity = vault_balance
        .saturating_sub(rent_exempt)
        .saturating_sub(clearing_house.total_liability);

    // Calculate SOL to return: lp_amount * available_liquidity / lp_supply
    let sol_amount_u128 = (lp_amount as u128)
        .checked_mul(available_liquidity as u128)
        .ok_or(FateSwapError::MathOverflow)?
        .checked_div(lp_supply as u128)
        .ok_or(FateSwapError::MathOverflow)?;

    let sol_amount = u64::try_from(sol_amount_u128).map_err(|_| FateSwapError::MathOverflow)?;

    // Ensure we're actually withdrawing some SOL
    require!(sol_amount > 0, FateSwapError::WithdrawTooSmall);

    // Ensure we have enough available liquidity
    require!(sol_amount <= available_liquidity, FateSwapError::InsufficientLiquidity);

    // Burn LP tokens from withdrawer
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.withdrawer_lp_account.to_account_info(),
                authority: ctx.accounts.withdrawer.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // Transfer SOL from vault to withdrawer via CPI (vault is system-owned PDA)
    let vault_bump = clearing_house.vault_bump;
    let vault_seeds = &[b"vault".as_ref(), &[vault_bump]];
    let vault_signer = &[&vault_seeds[..]];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.withdrawer.to_account_info(),
            },
            vault_signer,
        ),
        sol_amount,
    )?;

    // Track LP withdrawal accounting
    clearing_house.total_withdrawn = clearing_house
        .total_withdrawn
        .checked_add(sol_amount)
        .ok_or(FateSwapError::MathOverflow)?;
    clearing_house.lp_withdrawals_count = clearing_house
        .lp_withdrawals_count
        .checked_add(1)
        .ok_or(FateSwapError::MathOverflow)?;

    // Get updated balances for event (lp_mint needs reload after burn CPI)
    ctx.accounts.lp_mint.reload()?;

    let vault_balance_after = ctx.accounts.vault.lamports();
    let lp_supply_after = ctx.accounts.lp_mint.supply;

    // Emit event
    emit!(LiquidityWithdrawn {
        withdrawer: ctx.accounts.withdrawer.key(),
        lp_amount,
        sol_amount,
        vault_balance: vault_balance_after,
        lp_supply: lp_supply_after,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
