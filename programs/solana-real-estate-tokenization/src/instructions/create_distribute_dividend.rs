use anchor_lang::prelude::*;

use crate::ID;
use crate::landlord_emit;
use crate::events::DistributionCreated;
use crate::errors::LandLordErrors;
use crate::state::dividend_distributor::DividendDistributor;
use crate::state::platform_governor::PlatformGovernor;
use crate::utils::{spl_token_transfer, TokenTransferParams};
use crate::{assertions::cmp_pubkeys, constants::TOKEN_TREASURY_AUTHORITY_PDA_SEED};

use anchor_spl::token::{Mint, Token, TokenAccount, ID as TokenProgramID};

pub fn process_create_dividend_distribution_checkpoint(
    ctx: Context<CreateDividendCheckpoint>,
    root: [u8; 32],
    total_distribution_amount: u64,
) -> Result<()> {
    let owner = &ctx.accounts.owner;
    let token_program = &ctx.accounts.token_program;
    let treasury_token_account = &ctx.accounts.treasury_token_account;
    let owner_token_account = &ctx.accounts.owner_token_account;

    let seeds = [TOKEN_TREASURY_AUTHORITY_PDA_SEED];

    let derived_address = Pubkey::try_find_program_address(&seeds, &ID);

    if let Some((key, _)) = derived_address {
        if !cmp_pubkeys(&key, &treasury_token_account.owner) {
            return Err(LandLordErrors::TokenAccountOwnerMismatched.into());
        }
    
        spl_token_transfer(TokenTransferParams {
            source: owner_token_account.to_account_info(),
            destination: treasury_token_account.to_account_info(),
            authority: owner.to_account_info(),
            authority_signer_seeds: &[],
            token_program: token_program.to_account_info(),
            amount: total_distribution_amount,
        })?;
        
        let dividend_distributor = &mut ctx.accounts.dividend_distributor;
        let governor = &mut ctx.accounts.governor;
    
        let current_checkpoint = governor.update_dividend_checkpoints(1)?;
    
        dividend_distributor.init(
            current_checkpoint, 
            root, 
            owner.key(), 
            ctx.accounts.mint.key(),
            total_distribution_amount
        )?;

        landlord_emit!(
            DistributionCreated {
                checkpoint_id: current_checkpoint,
                distributor: dividend_distributor.key(),
                owner: owner.key(),
                root: hex::encode(root),
                total_distribution_amount
            }
        );
    } else {
        return Err(LandLordErrors::TokenTreasuryPDANotFound.into());
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct CreateDividendCheckpoint<'info> {
    // Dividend distributor
    #[account(init, payer = owner, space = DividendDistributor::LEN)]
    pub dividend_distributor: Account<'info, DividendDistributor>,

    // Asset owner
    #[account(mut)]
    pub owner: Signer<'info>,

    // #[account(
    //     seeds = [
    //         b"basket",
    //         mint_nft.key().as_ref(),
    //         owner.key().as_ref()    ,
    //         governor.key().as_ref()
    //     ],
    //     bump = asset_basket.bump,
    //     has_one = governor @LandLordErrors::GovernorMismatch,
    //     has_one = owner @LandLordErrors::NFTOwnerMismatch
    // )]
    // pub asset_basket: Box<Account<'info, AssetBasket>>,

    // /// CHECK: This only used for asset_basket seeds generate
    // pub mint_nft: UncheckedAccount<'info>,

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

    #[account(mut)]
    pub governor: Account<'info, PlatformGovernor>,

    pub token_program: Program<'info, Token>,

    // Solana native program
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
