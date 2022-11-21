// use anchor_lang::prelude::*;

// use crate::state::proposal::{Proposal, ProposalInstruction};
// use crate::state::Governor;

// /// Accounts for [govern::create_governor].
// #[derive(Accounts)]
// #[instruction(instructions: Vec<ProposalInstruction>)]
// pub struct CreateProposal<'info> {
//     /// The [Governor].
//     #[account(mut)]
//     pub governor: Account<'info, Governor>,
//     /// The [Proposal].
//     #[account(
//         init,
//         seeds = [
//             b"Lafomuo".as_ref(),
//             governor.key().as_ref(),
//             governor.proposal_count.to_le_bytes().as_ref()
//         ],
//         bump,
//         payer = payer,
//         space = Proposal::space(instructions),
//     )]
//     pub proposal: Box<Account<'info, Proposal>>,
//     /// Proposer of the proposal.
//     pub proposer: Signer<'info>,
//     /// Payer of the proposal.
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     /// System program.
//     pub system_program: Program<'info, System>,
// }