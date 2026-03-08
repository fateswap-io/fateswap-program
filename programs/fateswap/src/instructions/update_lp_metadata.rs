use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use crate::state::*;
use crate::errors::*;
use crate::instructions::create_lp_metadata::{TOKEN_METADATA_PROGRAM_ID, borsh_string};

#[derive(Accounts)]
pub struct UpdateLpMetadata<'info> {
    #[account(
        seeds = [b"clearing_house"],
        bump,
        has_one = authority,
    )]
    pub clearing_house: Account<'info, ClearingHouseState>,

    /// CHECK: Metadata PDA — validated by Metaplex program
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: LP mint authority PDA (update authority + signer for CPI)
    #[account(
        seeds = [b"lp_authority"],
        bump = clearing_house.lp_authority_bump,
    )]
    pub lp_authority: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Metaplex Token Metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<UpdateLpMetadata>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    require!(name.len() <= 32, FateSwapError::InvalidConfig);
    require!(symbol.len() <= 10, FateSwapError::InvalidConfig);
    require!(uri.len() <= 200, FateSwapError::InvalidConfig);

    // Build UpdateMetadataAccountV2 instruction data manually
    // Discriminator: 15
    let mut data = vec![15u8];

    // data: Option<DataV2> = Some(...)
    data.push(1); // Some

    // DataV2:
    borsh_string(&mut data, &name);
    borsh_string(&mut data, &symbol);
    borsh_string(&mut data, &uri);
    // seller_fee_basis_points: u16
    data.extend_from_slice(&0u16.to_le_bytes());
    // creators: Option<Vec<Creator>> = None
    data.push(0);
    // collection: Option<Collection> = None
    data.push(0);
    // uses: Option<Uses> = None
    data.push(0);

    // update_authority: Option<Pubkey> = None (keep current)
    data.push(0);

    // primary_sale_happened: Option<bool> = None
    data.push(0);

    // is_mutable: Option<bool> = Some(true)
    data.push(1); // Some
    data.push(1); // true

    let accounts = vec![
        AccountMeta::new(*ctx.accounts.metadata.key, false),
        AccountMeta::new_readonly(*ctx.accounts.lp_authority.key, true), // update authority, signer
    ];

    let ix = Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts,
        data,
    };

    // Sign with lp_authority PDA
    let seeds = &[b"lp_authority".as_ref(), &[ctx.accounts.clearing_house.lp_authority_bump]];
    let signer_seeds = &[&seeds[..]];

    invoke_signed(
        &ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.lp_authority.to_account_info(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
