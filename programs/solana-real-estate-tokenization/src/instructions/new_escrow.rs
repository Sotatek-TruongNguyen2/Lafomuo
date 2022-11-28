use anchor_lang::prelude::*;

use crate::landlord_emit;
use crate::state::fractional_token_escrow::FractionalTokenEscrow;
use crate::state::fractionalized_token_locker::FractionalizedTokenLocker;
use crate::events::NewEscrowEvent;
use crate::state::platform_governor::PlatformGovernor;

pub fn process_new_escrow(ctx: Context<NewEscrow>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    escrow.locker = ctx.accounts.locker.key();
    escrow.owner = ctx.accounts.escrow_owner.key();
    escrow.bump = *ctx.bumps.get("escrow").unwrap();

    // token account of the escrow is the ATA.
    escrow.hodl = anchor_spl::associated_token::get_associated_token_address(
        &escrow.key(),
        &ctx.accounts.locker.token_mint,
    );
    escrow.locked_amount = 0;
    escrow.suggested_price = 0;

    landlord_emit!(NewEscrowEvent {
        escrow: escrow.key(),
        escrow_owner: escrow.owner,
        locker: escrow.locker,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
/// Accounts for [locked_voter::new_escrow].
#[derive(Accounts)]
pub struct NewEscrow<'info> {
    /// [Locker].
    #[account(
        has_one = governor
    )]
    pub locker: Account<'info, FractionalizedTokenLocker>,

    pub governor: Account<'info, PlatformGovernor>,

    /// [Escrow].
    #[account(
        init,
        seeds = [
            b"escrow".as_ref(),
            locker.key().to_bytes().as_ref(),
            escrow_owner.key().to_bytes().as_ref()
        ],
        bump,
        payer = payer,
        space = FractionalTokenEscrow::LEN
    )]
    pub escrow: Account<'info, FractionalTokenEscrow>,

    /// CHECK: Authority of the [Escrow] to be created.
    pub escrow_owner: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}
