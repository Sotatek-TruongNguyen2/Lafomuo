use anchor_lang::prelude::*;  

use crate::state::{Governor, GovernanceParameters};
use crate::errors::LafomuoErrors;
use crate::events::GovernorCreateEvent;

pub fn create_governor(
    ctx: Context<CreateGovernor>,
    _bump: u8,
    electorate: Pubkey,
    params: GovernanceParameters,
) -> Result<()> {
    require!(
        params.timelock_delay_seconds >= 0,
        LafomuoErrors::TimeDelayMustBeGreaterThanZero
    );

    let governor = &mut ctx.accounts.governor;
    governor.base = ctx.accounts.base.key();
    governor.bump = *ctx.bumps.get("governor").unwrap();

    governor.proposal_count = 0;
    governor.electorate = electorate;
    governor.proposer = ctx.accounts.proposer.key();

    governor.params = params;

    emit!(GovernorCreateEvent {
        governor: governor.key(),
        electorate,
        proposer: ctx.accounts.proposer.key(),
        parameters: params,
    });

    Ok(())
}

/// Accounts for [govern::create_governor].
#[derive(Accounts)]
pub struct CreateGovernor<'info> {
    /// Base of the [Governor] key.
    pub base: Signer<'info>,
    /// Governor.
    #[account(
        init,
        seeds = [
            b"Lafomuo".as_ref(),
            base.key().as_ref()
        ],
        bump,
        payer = payer,
        space = Governor::LEN
    )]
    pub governor: Account<'info, Governor>,
    /// CHECK: The Smart Wallet.
    pub proposer: UncheckedAccount<'info>,
    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>
}
