use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::fractionalized_token_locker::FractionalizedTokenLocker;
use crate::state::fractional_token_escrow::FractionalTokenEscrow;
use crate::utils::{TokenTransferParams, spl_token_transfer};
use crate::escrow_seeds;
use crate::assertions::{assert_keys_equal, assert_keys_not_equal};
use crate::events::ExitEscrowEvent;
use crate::landlord_emit;
use crate::errors::LandLordErrors;

pub fn process_exit(
    ctx: Context<Exit>
) -> Result<()> {
    let locker = &mut ctx.accounts.locker;
    let escrow = &ctx.accounts.escrow;
    let escrow_hodl = &ctx.accounts.escrow_hodl;

    assert_keys_equal(&escrow.owner.key(), &ctx.accounts.escrow_owner.key())?;
    assert_keys_not_equal(&ctx.accounts.destination_tokens.key(), &escrow_hodl.key())?;

    let clock: Clock = Clock::get().unwrap();

    if locker.lock_end_time < clock.unix_timestamp {
        return Err(LandLordErrors::DistributionEndTimePassed.into());
    }

    let seeds: &[&[u8]] = escrow_seeds!(escrow);

    // transfer tokens from the escrow
    // if there are zero tokens in the escrow, short-circuit.
    if escrow.locked_amount > 0 {
        spl_token_transfer(TokenTransferParams {
            source: escrow_hodl.to_account_info(),
            destination: ctx.accounts.destination_tokens.to_account_info(),
            authority: escrow.to_account_info(),
            authority_signer_seeds: seeds,
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: escrow.locked_amount,
        })?;
    }

    // update the locker
    locker.locked_supply = locker.locked_supply.checked_sub(escrow.locked_amount).unwrap();

    landlord_emit!(ExitEscrowEvent {
        escrow_owner: escrow.owner,
        locker: locker.key(),
        locker_supply: locker.locked_supply,
        timestamp: Clock::get()?.unix_timestamp,
        released_amount: escrow.locked_amount,
    });

    Ok(())
}

/// Accounts for [locked_voter::exit].
#[derive(Accounts)]
pub struct Exit<'info> {
    /// The [Locker] being exited from.
    #[account(mut)]
    pub locker: Account<'info, FractionalizedTokenLocker>,

    /// The [Escrow] that is being closed.
    #[account(mut, has_one = locker, close = payer)]
    pub escrow: Account<'info, FractionalTokenEscrow>,

    /// Authority of the [Escrow].
    pub escrow_owner: Signer<'info>,
    /// Tokens locked up in the [Escrow].
    #[account(mut, constraint = escrow.hodl == escrow_hodl.key())]
    pub escrow_hodl: Account<'info, TokenAccount>,
    /// Destination for the tokens to unlock.
    #[account(mut)]
    pub destination_tokens: Account<'info, TokenAccount>,

    /// The payer to receive the rent refund.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}