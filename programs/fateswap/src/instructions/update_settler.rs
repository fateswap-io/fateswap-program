use anchor_lang::prelude::*;
use crate::state::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdateSettler<'info> {
    #[account(
        mut,
        seeds = [b"clearing_house"],
        bump,
        has_one = authority
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    pub authority: Signer<'info>,

    /// CHECK: New settler wallet (can be any pubkey)
    pub new_settler: AccountInfo<'info>,
}

pub fn handler(ctx: Context<UpdateSettler>) -> Result<()> {
    let clearing_house = &mut ctx.accounts.clearing_house;
    let clock = Clock::get()?;

    let old_settler = clearing_house.settler;
    clearing_house.settler = ctx.accounts.new_settler.key();

    emit!(SettlerUpdated {
        old_settler,
        new_settler: ctx.accounts.new_settler.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
