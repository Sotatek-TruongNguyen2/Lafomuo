use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, ID as TokenProgramID};

use crate::assertions::assert_is_ata;
use crate::constants::TOKEN_TREASURY_AUTHORITY_PDA_SEED;
use crate::errors::LandLordErrors;
use crate::events::DividendClaimed;
use crate::landlord_emit;
use crate::state::dividend_claimed::DividendClaimedDetails;
use crate::state::dividend_distributor::DividendDistributor;
use crate::state::fractional_token_escrow::FractionalTokenEscrow;
use crate::state::fractionalized_token_locker::FractionalizedTokenLocker;
use crate::ID;
use crate::{
    utils::{spl_token_transfer, TokenTransferParams},
};

pub fn process_claim_dividend(
    ctx: Context<ClaimDividendCheckpoint>,
    // amount: u64,
    // proof: Vec<[u8; 32]>
) -> Result<()> {
    let clock: Clock = Clock::get().unwrap();

    let escrow = &ctx.accounts.escrow;
    let locker = &ctx.accounts.locker;

    let token_program = &ctx.accounts.token_program;
    let claimed_dividend = &mut ctx.accounts.claimed_dividend;
    let dividend_distributor = &mut ctx.accounts.dividend_distributor;
    let claimer = &ctx.accounts.claimer;

    // // Verify the merkle proof.
    // let node = anchor_lang::solana_program::keccak::hashv(&[
    //     &dividend_distributor.key().to_bytes(),
    //     &claimer.key().to_bytes(),
    //     &amount.to_le_bytes()
    // ]);

    // let valid_claimer = verify(proof, dividend_distributor.root, node.0);

    // if !valid_claimer {
    //     return Err(LandLordErrors::InvalidMerkleProof.into());
    // }

    let available_claim_amount = (dividend_distributor.total_distribute_amount as u128).checked_mul(escrow.locked_amount as u128).unwrap().checked_div(locker.locked_supply as u128)
    .unwrap() as u64;

    if dividend_distributor.total_claimed + available_claim_amount > dividend_distributor.total_distribute_amount {
        return Err(LandLordErrors::ExceedsTotalDistributionAmount.into());
    }

    if claimed_dividend.total_claimed != 0 {
        return Err(LandLordErrors::DividendAlreadyClaimed.into());
    }

    let seeds = [TOKEN_TREASURY_AUTHORITY_PDA_SEED];

    let derived_address = Pubkey::try_find_program_address(&seeds, &ID);

    if let Some((key, bump)) = derived_address {
        assert_is_ata(
            &ctx.accounts
                .treasury_token_account
                .to_account_info()
                .to_account_info(),
            &key,
            &dividend_distributor.token_mint.key(),
        )?;

        spl_token_transfer(TokenTransferParams {
            source: ctx.accounts.treasury_token_account.to_account_info(),
            destination: ctx.accounts.claimer_token_account.to_account_info(),
            authority: ctx
                .accounts
                .treasury_token_account_authority
                .to_account_info(),
            authority_signer_seeds: &[TOKEN_TREASURY_AUTHORITY_PDA_SEED, &[bump]],
            token_program: token_program.to_account_info(),
            amount: available_claim_amount,
        })?;

        dividend_distributor.total_claimed = dividend_distributor
            .total_claimed
            .checked_add(available_claim_amount)
            .unwrap();

        claimed_dividend.checkpoint_id = dividend_distributor.id;
        claimed_dividend.claimer = claimer.key();
        claimed_dividend.token_mint = dividend_distributor.token_mint.key();
        claimed_dividend.total_claimed = available_claim_amount;
        claimed_dividend.last_claim_epoch = clock.unix_timestamp;

        landlord_emit!(DividendClaimed {
            checkpoint_id: claimed_dividend.checkpoint_id,
            distributor: dividend_distributor.key(),
            owner: ctx.accounts.claimer.key(),
            total_claimed: available_claim_amount,
            last_claimed_at: claimed_dividend.last_claim_epoch
        });
    } else {
        return Err(LandLordErrors::TokenTreasuryPDANotFound.into());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimDividendCheckpoint<'info> {
    // Dividend distributor
    #[account(mut, has_one = governor)]
    pub dividend_distributor: Box<Account<'info, DividendDistributor>>,

    /// CHECK: this account will be used to authorize dividend_distributor only
    pub governor: UncheckedAccount<'info>,

    /// The [Locker] being exited from.
    #[account(mut, has_one = dividend_distributor)]
    pub locker: Box<Account<'info, FractionalizedTokenLocker>>,

    /// The [Escrow] that is being closed. Sent the funds back to claimer
    #[account(mut, has_one = locker, close = claimer)]
    pub escrow: Box<Account<'info, FractionalTokenEscrow>>,

    // Asset owner
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
        mut,
        owner = TokenProgramID,
        constraint = claimer_token_account.mint == dividend_distributor.token_mint @LandLordErrors::TokenAccountMintMismatched,
        constraint = claimer_token_account.owner == claimer.key() @LandLordErrors::TokenAccountOwnerMismatched
    )]
    pub claimer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = TokenProgramID,
        constraint = treasury_token_account.mint == dividend_distributor.token_mint @LandLordErrors::TokenAccountMintMismatched,
        constraint = treasury_token_account.owner == treasury_token_account_authority.key() @LandLordErrors::TokenAccountOwnerMismatched
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// CHECK: this will be checked through CPI call
    pub treasury_token_account_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = claimer,
        seeds = [
            b"claim_dividend",
            dividend_distributor.key().as_ref(),
            claimer.key().as_ref()    ,
        ],
        space = DividendClaimedDetails::LEN,
        bump,
    )]
    pub claimed_dividend: Box<Account<'info, DividendClaimedDetails>>,
    // Token program
    pub token_program: Program<'info, Token>,

    // Solana native program
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
