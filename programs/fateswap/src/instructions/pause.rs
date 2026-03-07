use anchor_lang::prelude::*;
use crate::state::*;
use crate::events::*;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        seeds = [b"clearing_house"],
        bump,
        has_one = authority
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Pause>, paused: bool) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    clearing_house.paused = paused;

    emit!(Paused {
        authority: ctx.accounts.authority.key(),
        paused,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
