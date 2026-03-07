use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use crate::state::*;
use crate::errors::*;
use crate::events::*;
use crate::math::*;

#[derive(Accounts)]
pub struct DepositSol<'info> {
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
        seeds = [b"lp_authority"],
        bump = clearing_house.lp_authority_bump
    )]
    /// CHECK: LP mint authority PDA
    pub lp_authority: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = lp_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_lp_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    // Check not paused
    require!(!clearing_house.paused, FateSwapError::Paused);

    // Check amount not zero
    require!(amount > 0, FateSwapError::ZeroAmount);

    // Get current vault balance and LP supply
    let vault_balance_before = ctx.accounts.vault.lamports();
    let lp_supply_before = ctx.accounts.lp_mint.supply;

    // Calculate effective balance for LP pricing (vault - rent - liability)
    let rent_exempt = Rent::get()?.minimum_balance(8); // 8-byte discriminator
    let effective_balance = vault_balance_before
        .saturating_sub(rent_exempt)
        .saturating_sub(clearing_house.total_liability);

    // Calculate LP tokens to mint
    let lp_to_mint: u64;
    let is_first_deposit = lp_supply_before == 0;

    if is_first_deposit {
        // First deposit: mint 1:1 minus MINIMUM_LIQUIDITY
        require!(amount > MINIMUM_LIQUIDITY, FateSwapError::DepositTooSmall);
        lp_to_mint = amount - MINIMUM_LIQUIDITY;
    } else {
        // Subsequent deposits: proportional to current pool share
        // lp_to_mint = amount * lp_supply / effective_balance
        let lp_u128 = (amount as u128)
            .checked_mul(lp_supply_before as u128)
            .ok_or(FateSwapError::MathOverflow)?
            .checked_div(effective_balance as u128)
            .ok_or(FateSwapError::MathOverflow)?;

        lp_to_mint = u64::try_from(lp_u128).map_err(|_| FateSwapError::MathOverflow)?;
    }

    // Ensure we're actually minting some LP
    require!(lp_to_mint > 0, FateSwapError::DepositTooSmall);

    // Transfer SOL from depositor to vault
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    // Mint LP tokens to depositor
    let lp_authority_seeds = &[b"lp_authority".as_ref(), &[clearing_house.lp_authority_bump]];
    let signer_seeds = &[&lp_authority_seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.depositor_lp_account.to_account_info(),
                authority: ctx.accounts.lp_authority.to_account_info(),
            },
            signer_seeds,
        ),
        lp_to_mint,
    )?;

    // Track LP deposit accounting
    clearing_house.total_deposited = clearing_house
        .total_deposited
        .checked_add(amount)
        .ok_or(FateSwapError::MathOverflow)?;
    clearing_house.lp_deposits_count = clearing_house
        .lp_deposits_count
        .checked_add(1)
        .ok_or(FateSwapError::MathOverflow)?;

    // Get updated balances for event (lp_mint needs reload after mint_to CPI)
    ctx.accounts.lp_mint.reload()?;

    let vault_balance_after = ctx.accounts.vault.lamports();
    let lp_supply_after = ctx.accounts.lp_mint.supply;

    // Emit event
    emit!(LiquidityDeposited {
        depositor: ctx.accounts.depositor.key(),
        sol_amount: amount,
        lp_amount: lp_to_mint,
        vault_balance: vault_balance_after,
        lp_supply: lp_supply_after,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
