use crate::state::fractional_token_escrow::FractionalTokenEscrow;
use crate::state::fractionalized_token_locker::FractionalizedTokenLocker;
use crate::utils::{spl_token_transfer, TokenTransferParams};
use crate::events::LockEvent;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub fn process_escrow_lock(ctx: Context<Lock>, amount: u64) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    let locker = &mut ctx.accounts.locker;

    // transfer tokens to the escrow
    // if amount is 0, we can skip this call.
    // One would lock 0 tokens at a duration to be able to refresh their existing lockup.
    if amount > 0 {
        spl_token_transfer(TokenTransferParams {
            source: ctx.accounts.source_tokens.to_account_info(),
            destination: ctx.accounts.escrow_token_hodl.to_account_info(),
            authority: ctx.accounts.escrow_owner.to_account_info(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount,
        })?;
    }

    escrow.locked_amount = escrow.locked_amount.checked_add(amount).unwrap();
    locker.locked_supply = locker.locked_supply.checked_add(amount).unwrap();

    emit!(LockEvent {
        locker: locker.key(),
        locker_supply: locker.locked_supply,
        escrow_owner: escrow.owner,
        token_mint: locker.token_mint,
        amount
    });

    Ok(())
}

/// Accounts for [locked_voter::lock].
#[derive(Accounts)]
pub struct Lock<'info> {
    /// [Locker].
    #[account(mut)]
    pub locker: Account<'info, FractionalizedTokenLocker>,

    /// [Escrow].
    #[account(mut, has_one = locker)]
    pub escrow: Account<'info, FractionalTokenEscrow>,

    /// Token account held by the [Escrow].
    #[account(
        mut,
        constraint = escrow.hodl == escrow_token_hodl.key()
    )]
    pub escrow_token_hodl: Account<'info, TokenAccount>,

    /// Authority of the [Escrow] and [Self::source_tokens].
    pub escrow_owner: Signer<'info>,

    /// The source of deposited tokens.
    #[account(mut)]
    pub source_tokens: Account<'info, TokenAccount>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}
