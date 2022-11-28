use anchor_lang::prelude::*;

use crate::ID;
use crate::landlord_emit;
use crate::state::setting::LafomuoSetting;
use crate::assertions::assert_is_ata;
use crate::events::DistributionCreated;
use crate::errors::LandLordErrors;
use crate::state::asset_basket::AssetBasket;
use crate::state::dividend_distributor::DividendDistributor;
use crate::state::platform_governor::PlatformGovernor;
use crate::utils::{spl_token_transfer, TokenTransferParams};
use crate::constants::TOKEN_TREASURY_AUTHORITY_PDA_SEED;
use crate::state::fractionalized_token_locker::FractionalizedTokenLocker;

use anchor_spl::token::{Mint, Token, TokenAccount, ID as TokenProgramID};

pub fn process_create_dividend_distribution_checkpoint(
    ctx: Context<CreateDividendCheckpoint>,
    total_distribution_amount: u64,
) -> Result<()> {
    let setting  = &ctx.accounts.setting;
    let owner = &ctx.accounts.owner;
    let token_program = &ctx.accounts.token_program;
    let treasury_token_account = &ctx.accounts.treasury_token_account;
    let owner_token_account = &ctx.accounts.owner_token_account;
    let asset_basket = &mut ctx.accounts.asset_basket;

     // Get lock end time for this distributor
     let clock: Clock = Clock::get().unwrap();
     let dividend_distributor = &mut ctx.accounts.dividend_distributor;
     let start_distribution_at = clock.unix_timestamp.checked_add(setting.pre_lock_before_distribution).unwrap();

     dividend_distributor.init(
        asset_basket.total_distribution_checkpoint , 
        ctx.accounts.governor.key(),
        owner.key(), 
        ctx.accounts.mint.key(),
        total_distribution_amount
    )?;

    let fractionalize_token_locker = &mut ctx.accounts.fractionalize_token_locker;
    fractionalize_token_locker.bump = *ctx.bumps.get("fractionalize_token_locker").unwrap();
    fractionalize_token_locker.dividend_distributor = dividend_distributor.key();
    fractionalize_token_locker.governor = ctx.accounts.governor.key();
    fractionalize_token_locker.locked_supply = 0;
    fractionalize_token_locker.basket_id = asset_basket.basket_id;
    fractionalize_token_locker.lock_end_time = start_distribution_at;
    fractionalize_token_locker.token_mint = asset_basket.asset_tokenize.token_mint;

    let seeds = [TOKEN_TREASURY_AUTHORITY_PDA_SEED];

    let derived_treasury_address = Pubkey::try_find_program_address(&seeds, &ID);

    if let Some((key, _)) = derived_treasury_address {
        assert_is_ata(&treasury_token_account.to_account_info(), &key, &ctx.accounts.mint.key())?;

        spl_token_transfer(TokenTransferParams {
            source: owner_token_account.to_account_info(),
            destination: treasury_token_account.to_account_info(),
            authority: owner.to_account_info(),
            authority_signer_seeds: &[],
            token_program: token_program.to_account_info(),
            amount: total_distribution_amount,
        })?;
        
        let governor = &mut ctx.accounts.governor;  
        // Update global total dividend checkpoints
        governor.update_dividend_checkpoints(1)?;

        landlord_emit!(
            DistributionCreated {
                checkpoint_id: asset_basket.total_distribution_checkpoint,
                distributor: dividend_distributor.key(),
                owner: owner.key(),
                // root: hex::encode(root),
                total_distribution_amount,
                start_distribution_at,
                locker: fractionalize_token_locker.key()
            }
        );

        asset_basket.total_distribution_checkpoint = asset_basket.total_distribution_checkpoint.checked_add(1).unwrap(); 
    } else {
        return Err(LandLordErrors::TokenTreasuryPDANotFound.into());
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct CreateDividendCheckpoint<'info> {
    // Dividend distributor
    #[account(init, payer = owner, space = DividendDistributor::LEN)]
    pub dividend_distributor: Box<Account<'info, DividendDistributor>>,

    /// [Locker].
    #[account(
        init,
        seeds = [
            b"locker",
            governor.key().as_ref(),
            [asset_basket.total_distribution_checkpoint as u8].as_ref(),
            [asset_basket.basket_id as u8].as_ref()
        ],
        bump,
        payer = owner,
        space = FractionalizedTokenLocker::LEN
    )]
    pub fractionalize_token_locker: Box<Account<'info, FractionalizedTokenLocker>>,

    #[account(
        mut,
        seeds = [   
            b"basket",
            asset_basket.asset_id.key().as_ref(),
            owner.key().as_ref()    ,
            governor.key().as_ref(),
            [asset_basket.basket_id as u8].as_ref()
        ],
        bump = asset_basket.bump,
    )]
    pub asset_basket: Box<Account<'info, AssetBasket>>,

    // Asset owner
    #[account(mut)]
    pub owner: Signer<'info>,

    // Must be signed by big guardian to authorize asset issuing
    pub big_guardian: Signer<'info>,

    pub setting: Box<Account<'info, LafomuoSetting>>,

    #[account(
        mut,
        has_one = big_guardian,
        has_one = setting
    )]
    pub governor: Account<'info, PlatformGovernor>,

    #[account(
        mut,
        owner = TokenProgramID
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        owner = TokenProgramID,
        constraint = owner_token_account.mint == mint.key() @LandLordErrors::TokenAccountMintMismatched,
        constraint = owner_token_account.owner == owner.key() @LandLordErrors::TokenAccountOwnerMismatched
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = TokenProgramID,
        constraint = treasury_token_account.mint == mint.key() @LandLordErrors::TokenAccountMintMismatched
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    // Solana native program
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
