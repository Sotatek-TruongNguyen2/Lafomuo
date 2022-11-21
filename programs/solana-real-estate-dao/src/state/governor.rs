use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct Governor {
    /// Base.
    pub base: Pubkey,
    /// Bump seed
    pub bump: u8,

    /// The total number of [Proposal]s
    pub proposal_count: u64,
    /// The voting body associated with the Governor.
    /// This account is responsible for handling vote proceedings, such as:
    /// - activating proposals
    /// - setting the number of votes per voter
    pub electorate: Pubkey,
    /// The public key of the [smart_wallet::SmartWallet] account.
    /// This smart wallet executes proposals.
    pub proposer: Pubkey,

    /// Governance parameters.
    pub params: GovernanceParameters,
}

impl Governor {
    /// Number of bytes in a [Governor].
    pub const LEN: usize = DISCRIMINATOR_LENGTH + PUBLIC_KEY_LENGTH + 1 + 8 + PUBLIC_KEY_LENGTH * 2 + GovernanceParameters::LEN;
}

/// Governance parameters.
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct GovernanceParameters {
    /// The delay before voting on a proposal may take place, once proposed, in seconds
    pub voting_delay: u64,
    /// The duration of voting on a proposal, in seconds
    pub voting_period: u64,
    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: u64,
    /// The timelock delay of the DAO's created proposals.
    pub timelock_delay_seconds: i64,
}

impl GovernanceParameters {
    /// Number of bytes in a [GovernanceParameters].
    pub const LEN: usize = 8 * 4;
}


// /// Metadata about a proposal.
// #[account]
// #[derive(Debug, Default)]
// pub struct ProposalMeta {
//     /// The [Proposal].
//     pub proposal: Pubkey,
//     /// Title of the proposal.
//     pub title: String,
//     /// Link to a description of the proposal.
//     pub description_link: String,
// }

// /// A [Vote] is a vote made by a `voter` by an `electorate`.
// #[account]
// #[derive(Debug, Default)]
// pub struct Vote {
//     /// The proposal being voted on.
//     pub proposal: Pubkey,
//     /// The voter.
//     pub voter: Pubkey,
//     /// Bump seed
//     pub bump: u8,

//     /// The side of the vote taken.
//     pub side: u8,
//     /// The number of votes this vote holds.
//     pub weight: u64,
// }

// impl Vote {
//     /// Number of bytes in a [Vote].
//     pub const LEN: usize = DISCRIMINATOR_LENGTH + PUBLIC_KEY_LENGTH * 2 + 1 + 1 + 8;
// }

// /// Instruction.
// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
// pub struct ProposalInstruction {
//     /// Pubkey of the instruction processor that executes this instruction
//     pub program_id: Pubkey,
//     /// Metadata for what accounts should be passed to the instruction processor
//     pub keys: Vec<ProposalAccountMeta>,
//     /// Opaque data passed to the instruction processor
//     pub data: Vec<u8>,
// }

// impl ProposalInstruction {
//     /// Space that a [ProposalInstruction] takes up.
//     pub fn space(&self) -> usize {
//         std::mem::size_of::<Pubkey>()
//             + (self.keys.len() as usize) * std::mem::size_of::<AccountMeta>()
//             + (self.data.len() as usize)
//     }
// }

// /// Account metadata used to define Instructions
// #[derive(AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Copy, Clone)]
// pub struct ProposalAccountMeta {
//     /// An account's public key
//     pub pubkey: Pubkey,
//     /// True if an Instruction requires a Transaction signature matching `pubkey`.
//     pub is_signer: bool,
//     /// True if the `pubkey` can be loaded as a read-write account.
//     pub is_writable: bool,
// }