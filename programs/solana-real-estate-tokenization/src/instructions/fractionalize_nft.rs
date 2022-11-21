use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, program_option::COption};

use anchor_spl::token::spl_token::instruction::{set_authority, AuthorityType};
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount, ID as TokenProgramID};
use arrayref::{array_ref, array_refs};

use crate::events::NewLockerEvent;
use crate::assertions::assert_is_ata;
use crate::constants::TOKEN_TREASURY_AUTHORITY_PDA_SEED;
use crate::errors::LandLordErrors;
use crate::state::asset_basket::AssetBasket;
use crate::state::asset_locker::AssetLocker;
use crate::state::platform_governor::PlatformGovernor;
use crate::utils::{spl_token_transfer, TokenTransferParams};
use crate::ID;

pub fn process_fractionalize_asset(
    ctx: Context<FractionalizeNFT>,
    total_supply: u64,
) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let mint_account_info = mint.to_account_info();
    let mint_authority_info = &ctx.accounts.owner.to_account_info();

    let mint_nft = &ctx.accounts.mint_nft;

    let treasury_nft_token_account = &ctx.accounts.treasury_nft_token_account;
    let (derived_treasury_address, _) = Pubkey::try_find_program_address(&[TOKEN_TREASURY_AUTHORITY_PDA_SEED], &ID).unwrap();

    // PDA check
    assert_is_ata(&&treasury_nft_token_account.to_account_info(), &derived_treasury_address, &mint_nft.key())?;

    let asset_locker = &mut ctx.accounts.asset_locker;
    asset_locker.bump = *ctx.bumps.get("asset_locker").unwrap();
    asset_locker.base = ctx.accounts.owner.key();
    asset_locker.governor = ctx.accounts.governor.key();
    asset_locker.locked_supply = 0;


    emit!(NewLockerEvent {
        governor: ctx.accounts.governor.key(),
        locker: asset_locker.key(),
        token_mint: mint_nft.key(),
        asset_id: ctx.accounts.asset_basket.asset_id,
        basket_id: ctx.accounts.asset_basket.basket_id
    });

    let asset_basket = &mut ctx.accounts.asset_basket;

    asset_basket.fractionalize(total_supply, mint)?;

    let big_guardian = &ctx.accounts.big_guardian;
    let big_guardian_account_info = big_guardian.to_account_info();

    mint_token(&ctx, total_supply)?;

    transfer_mint_authority(
        &big_guardian.key(),
        &big_guardian_account_info,
        &mint_account_info,
        mint_authority_info,
        &ctx.accounts.token_program.to_account_info(),
    )?;

    spl_token_transfer(TokenTransferParams {
        source: ctx.accounts.owner_nft_token_account.to_account_info(),
        destination: treasury_nft_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
        authority_signer_seeds: &[],
        token_program: ctx.accounts.token_program.to_account_info(),
        amount: 1,
    })?;

    Ok(())
}

fn mint_token<'a>(ctx: &'a Context<FractionalizeNFT>, total_supply: u64) -> Result<&'a Pubkey> {
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let token_mint = ctx.accounts.mint.to_account_info();
    let token_mint_id = token_mint.key;
    let token_account_info = &ctx.accounts.token_account.to_account_info();

    assert_is_ata(
        token_account_info,
        &ctx.accounts.owner.key(),
        &token_mint_id,
    )?;

    let cpi_accounts = MintTo {
        mint: token_mint,
        to: token_account_info.clone(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    let mint_amount = total_supply;
    anchor_spl::token::mint_to(cpi_ctx, mint_amount)?;

    Ok(token_mint_id)
}

pub fn transfer_mint_authority<'a>(
    governor: &Pubkey,
    governor_account_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
) -> Result<()> {
    msg!("Setting mint authority");
    let accounts = &[
        mint_authority_info.clone(),
        mint_info.clone(),
        token_program_info.clone(),
        governor_account_info.clone(),
    ];
    invoke_signed(
        &set_authority(
            token_program_info.key,
            mint_info.key,
            Some(governor),
            AuthorityType::MintTokens,
            mint_authority_info.key,
            &[mint_authority_info.key],
        )
        .unwrap(),
        accounts,
        &[],
    )?;
    msg!("Setting freeze authority");
    let freeze_authority = get_mint_freeze_authority(mint_info)?;
    if freeze_authority.is_some() {
        invoke_signed(
            &set_authority(
                token_program_info.key,
                mint_info.key,
                Some(governor),
                AuthorityType::FreezeAccount,
                mint_authority_info.key,
                &[mint_authority_info.key],
            )
            .unwrap(),
            accounts,
            &[],
        )?;
        msg!("Finished setting freeze authority");
    } else {
        return Err(LandLordErrors::NoFreezeAuthoritySet.into());
    }

    Ok(())
}

pub fn get_mint_freeze_authority(account_info: &AccountInfo) -> Result<COption<Pubkey>> {
    let data = account_info.try_borrow_data().unwrap();
    let authority_bytes = array_ref![data, 36 + 8 + 1 + 1, 36];

    unpack_coption_key(authority_bytes)
}

/// Unpacks COption from a slice, taken from token program
fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(LandLordErrors::InvalidAccountData.into()),
    }
}

#[derive(Accounts)]
pub struct FractionalizeNFT<'info> {
    // Asset owner
    #[account(mut)]
    pub owner: Signer<'info>,
    // Must be signed by big guardian to authorize asset issuing
    pub big_guardian: Signer<'info>,

    /// [Locker].
    #[account(
        init,
        seeds = [
            b"locker",
            governor.key().as_ref(),
            [asset_basket.basket_id as u8].as_ref()
        ],
        bump,
        payer = owner,
        space = AssetLocker::LEN
    )]
    pub asset_locker: Box<Account<'info, AssetLocker>>,

    #[account(
        mut,
        seeds = [
            b"basket",
            mint_nft.key().as_ref(),
            owner.key().as_ref(),
            governor.key().as_ref(),
            [asset_basket.basket_id as u8].as_ref()
        ],
        bump = asset_basket.bump,
        has_one = governor @LandLordErrors::GovernorMismatch,
        has_one = owner @LandLordErrors::NFTOwnerMismatch
    )]
    pub asset_basket: Box<Account<'info, AssetBasket>>,

    /// CHECK: This will be checked using CPI
    #[account(mut)]
    pub treasury_nft_token_account: UncheckedAccount<'info>,

    /// CHECK: This only used for asset_basket seeds generate
    pub mint_nft: UncheckedAccount<'info>,

    /// CHECK: This only used for asset_basket seeds generate
    #[account(
        mut,
        owner = TokenProgramID,
        constraint = owner_nft_token_account.mint == mint_nft.key() @LandLordErrors::TokenAccountMintMismatched,
        constraint = token_account.owner == owner.key() @LandLordErrors::TokenAccountOwnerMismatched
    )]
    pub owner_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = TokenProgramID
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        owner = TokenProgramID,
        constraint = token_account.mint == mint.key() @LandLordErrors::TokenAccountMintMismatched,
        constraint = token_account.owner == owner.key() @LandLordErrors::TokenAccountOwnerMismatched
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = big_guardian
    )]
    pub governor: Account<'info, PlatformGovernor>,

    pub token_program: Program<'info, Token>,

    // Solana native program
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
