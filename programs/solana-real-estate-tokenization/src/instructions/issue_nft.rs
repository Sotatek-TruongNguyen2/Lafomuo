use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{Mint, Token, MintTo, ID as TokenProgramID, TokenAccount};
use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2};

use crate::landlord_emit;
use crate::events::{AssetIssuance};
use crate::errors::LandLordErrors;
use crate::state::asset_basket::{AssetBasket};
use crate::state::platform_governor::PlatformGovernor;
use crate::utils::{spl_token_transfer, TokenTransferParams};
use crate::assertions::assert_is_ata;

pub fn process_issue_asset<'a>(ctx: Context<'_, '_, '_, 'a, IssueAsset<'a>>, uri: String, title: String) -> Result<()> {
    let governor = &mut ctx.accounts.governor;
    let treasury = &ctx.accounts.treasury;
    let token_program = &ctx.accounts.token_program;
    let owner = &ctx.accounts.owner;
    let mint = &ctx.accounts.mint;

    if governor.is_mutable {
        governor.increase_total_minted(1)?;
    } else {
        return Err(LandLordErrors::ImmutableGovernor.into());
    }

    // =====  Do asset basket initialization =====
    let asset_basket = &mut ctx.accounts.asset_basket;

    asset_basket.init(
        governor.total_assets_minted,
        *ctx.bumps.get("asset_basket").unwrap(),
        ctx.accounts.metadata.key(),
        governor.key(),
        owner.key(),
        mint.key()
    )?;

    let token_mint = governor.minting_protocol_token;

    // Take fee from asset owner when issuing NFT
    if token_mint.is_some() { 
        // If platform payment token is SPL ( not SOL native )
        if !ctx.remaining_accounts.is_empty() {
            let token_account_info = &ctx.remaining_accounts[0];
            let token_account = assert_is_ata(token_account_info, &owner.key(), &mint.key())?;

            if token_account.amount < governor.minting_protocol_price {
                return Err(LandLordErrors::NotEnoughTokens.into());
            }

            spl_token_transfer(TokenTransferParams {
                source: token_account_info.clone(),
                destination: treasury.to_account_info(),
                authority: owner.to_account_info(),
                authority_signer_seeds: &[],
                token_program: token_program.to_account_info(),
                amount: governor.minting_protocol_price,
            })?;
        }
    } else {
        if owner.lamports() < governor.minting_protocol_price {
            return Err(LandLordErrors::NotEnoughSOL.into());
        }

        invoke(
            &system_instruction::transfer(&owner.key(), &treasury.key(), governor.minting_protocol_price),
            &[
                owner.to_account_info(),
                treasury.to_account_info(),
                token_program.to_account_info(),
            ],
        )?;
    }

    landlord_emit!(
        AssetIssuance {
            asset_id: mint.key(),
            owner: owner.key(),
            owner_pda: asset_basket.key(),
            iat: asset_basket.iat,
            master_edition: ctx.accounts.master_edition.key(),
            metadata: ctx.accounts.metadata.key(),
            asset_token_account: ctx.accounts.token_account.key()
        }
    );

    msg!("Initializing mint");
    let token_mint_id = mint_token(&ctx)?;
    msg!("Minted token id: {}", token_mint_id);

    msg!("Initializing metadata account");
    create_metadata_accounts(&ctx, uri, title)?;
    msg!("Metadata account created !!!");

    msg!("Initializing master edition nft");
    create_master_edition(&ctx)?;
    msg!("Master edition nft minted !!!");

    Ok(())
}

fn mint_token<'a>(ctx: &'a Context<IssueAsset>) -> Result<&'a Pubkey> {
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let token_mint = ctx.accounts.mint.to_account_info();
    let token_mint_id = token_mint.key;
    let cpi_accounts = MintTo {
        mint: token_mint,
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    let mint_amount = 1;
    anchor_spl::token::mint_to(cpi_ctx, mint_amount)?;

    Ok(token_mint_id)
}

fn create_metadata_accounts<'a>(
    ctx: &'a Context<IssueAsset>,
    uri: String,
    title: String,
) -> Result<()> {
    let symbol = &ctx.accounts.governor.symbol;
    let big_guardian = ctx.accounts.governor.big_guardian;
    let land_owner = &ctx.accounts.owner;
    // creator
    let creator = vec![
        mpl_token_metadata::state::Creator {
            address: land_owner.key(),
            verified: false,
            share: 90,
        },
        mpl_token_metadata::state::Creator {
            address: big_guardian,
            verified: false,
            share: 10,
        }
    ];
    // account info
    let account_info = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];
    invoke(
        &create_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.mint_authority.key(),
            title,
            symbol.clone(),
            uri,
            Some(creator),
            1,
            true,
            false,
            None,
            None,
        ),
        account_info.as_slice(),
    )?;

    Ok(())
}

fn create_master_edition(ctx: &Context<IssueAsset>) -> Result<()> {
    // master edition info
    let master_edition_infos = vec![
        ctx.accounts.master_edition.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];

    invoke(
        &create_master_edition_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.master_edition.key(),
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint_authority.key(),
            Some(0),
        ),
        master_edition_infos.as_slice(),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct IssueAsset<'info> {
    // With the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.

    /// CHECK: account checked in CPI
    #[account(mut)]
    metadata: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    master_edition: UncheckedAccount<'info>,
 
    /// CHECK: Account treasury to store all tokens
    #[account(mut)]
    treasury: UncheckedAccount<'info>,
    
    // Asset owner mint and update authority. Needs to be transferred to Master edition or Edition account
    pub mint_authority: Signer<'info>,
    pub update_authority: Signer<'info>,
    
    // Asset owner 
    #[account(mut)]
    pub owner: Signer<'info>,

    // Must be signed by big guardian to authorize asset issuing
    pub big_guardian: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [
            b"basket",
            mint.key().as_ref(),
            owner.key().as_ref(),
            governor.key().as_ref(),
            [(governor.total_assets_minted + 1) as u8].as_ref()
        ],
        space = AssetBasket::LEN,
        bump,
    )]
    pub asset_basket: Box<Account<'info, AssetBasket>>,

    #[account(
        mut,
        owner = TokenProgramID
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = treasury,
        has_one = big_guardian
    )]
    pub governor: Account<'info, PlatformGovernor>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    // Solana native program
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
