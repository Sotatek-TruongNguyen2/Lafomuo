use anchor_lang::prelude::*;
use num_traits::cast::ToPrimitive;

use crate::events::ProposalCreateEvent;
use crate::state::proposal::{LafomuoInstruction, Proposal};
use crate::state::Governor;

/// Creates a [Proposal].
/// This may be called by anyone, since the [Proposal] does not do anything until
/// it is activated in [activate_proposal].
pub fn process_create_proposal(
    ctx: Context<CreateProposal>,
    _bump: u8,
    instructions: Vec<LafomuoInstruction>,
) -> Result<()> {
    let governor = &mut ctx.accounts.governor;

    let proposal = &mut ctx.accounts.proposal;
    proposal.governor = governor.key();
    proposal.index = governor.proposal_count;
    proposal.bump = *ctx.bumps.get("proposal").unwrap();

    proposal.proposer = ctx.accounts.proposer.key();

    proposal.quorum_votes = governor.params.quorum_votes;

    let now = Clock::get()?.unix_timestamp;

    proposal.created_at = now;
    proposal.canceled_at = 0;

    proposal.activated_at = governor
        .params
        .voting_delay
        .to_i64()
        .and_then(|v: i64| now.checked_add(v))
        .unwrap();

    proposal.voting_ends_at = governor
        .params
        .voting_period
        .to_i64()
        .and_then(|v: i64| proposal.activated_at.checked_add(v))
        .unwrap();

    proposal.instructions = instructions.clone();
    governor.proposal_count += 1;

    emit!(ProposalCreateEvent {
        governor: governor.key(),
        proposal: proposal.key(),
        index: proposal.index,
        instructions,
    });

    Ok(())
}

/// Accounts for [govern::create_governor].
#[derive(Accounts)]
#[instruction(instructions: Vec<LafomuoInstruction>)]
pub struct CreateProposal<'info> {
    /// The [Governor].
    #[account(mut)]
    pub governor: Account<'info, Governor>,
    /// The [Proposal].
    #[account(
        init,
        seeds = [
            b"Lafomuo".as_ref(),
            governor.key().as_ref(),
            governor.proposal_count.to_le_bytes().as_ref()
        ],
        bump,
        payer = payer,
        space = Proposal::space(instructions),
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    /// Proposer of the proposal.
    pub proposer: Signer<'info>,
    /// Payer of the proposal.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}
