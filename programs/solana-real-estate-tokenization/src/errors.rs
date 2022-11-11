use anchor_lang::error_code;

#[error_code]
pub enum LandLordErrors {
    #[msg("Account has not initialized yet!")]
    AccountNotInitialized,
    #[msg("Not able to unpack account!")]
    NotAbleToUnpackAccount,
    #[msg("Invalid account owner!")]
    IncorrectOwner,
    #[msg("Platform has no authorities at all!")]
    PlatformHasNoAuthorities,
    NotOwnedBySPLProgram,
    TokenAccountMintMismatched,
    TokenAccountOwnerMismatched,
    TokenTransferFailed,
    PublicKeyMismatch,
    NotEnoughTokens,
    NotEnoughSOL,
    MintingAmountCantBeZero,
    ImmutableGovernor
}