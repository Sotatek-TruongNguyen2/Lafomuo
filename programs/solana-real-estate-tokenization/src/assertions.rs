use crate::errors::LandLordErrors;
use anchor_lang::solana_program::{
    account_info::AccountInfo,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    program_memory::sol_memcmp
};
use anchor_spl::token::spl_token;

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn assert_initialized<L: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<L, LandLordErrors> {
    let account = L::unpack_unchecked(&account_info.data.borrow());

    if account.is_err() {
        return Err(LandLordErrors::NotAbleToUnpackAccount.into());
    }

    let unwrapped_account = account.unwrap();
 
    if !unwrapped_account.is_initialized() {
        Err(LandLordErrors::AccountNotInitialized.into())
    } else {
        Ok(unwrapped_account)
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<(), LandLordErrors> {
    if !cmp_pubkeys(account.owner, owner) {
        Err(LandLordErrors::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

pub fn assert_is_ata(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Result<spl_token::state::Account, LandLordErrors> {
    assert_owned_by(ata, &spl_token::id())?;
    let ata_account: spl_token::state::Account = assert_initialized(ata)?;
    assert_keys_equal(&ata_account.owner, wallet)?;
    assert_keys_equal(&ata_account.mint, mint)?;
    assert_keys_equal(&anchor_spl::associated_token::get_associated_token_address(wallet, mint), ata.key)?;
    Ok(ata_account)
}

pub fn assert_keys_equal(key1: &Pubkey, key2: &Pubkey) -> Result<(), LandLordErrors> {
    if !cmp_pubkeys(key1, key2) {
        return Err(LandLordErrors::PublicKeyMismatch)
    } else {
        Ok(())
    }
}