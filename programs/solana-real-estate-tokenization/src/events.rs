use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use std::io::Write;

/// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
///
/// Warning: This will allocate large buffers on the heap which will never be released as Solana
/// uses a simple bump allocator where free() is a noop. Since the max heap size is limited
// (32k currently), calling this multiple times can lead to memory allocation failures.
#[macro_export]
macro_rules! landlord_emit {
    ($e:expr) => {
        msg!("landlord-log");
        emit!($e);
    };
}

/// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
///
/// Warning: This stores intermediate results on the stack, which must have 2*N+ free bytes.
/// This function will panic if the generated event does not fit the buffer of size N.
pub fn mango_emit_stack<T: AnchorSerialize + Discriminator, const N: usize>(event: T) {
    let mut data_buf = [0u8; N];
    let mut out_buf = [0u8; N];

    mango_emit_buffers(event, &mut data_buf[..], &mut out_buf[..])
}

/// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
///
/// This function will write intermediate data to data_buf and out_buf. The buffers must be
/// large enough to hold this data, or the function will panic.
pub fn mango_emit_buffers<T: AnchorSerialize + Discriminator>(
    event: T,
    data_buf: &mut [u8],
    out_buf: &mut [u8],
) {
    let mut data_writer = std::io::Cursor::new(data_buf);
    data_writer
        .write_all(&<T as Discriminator>::discriminator())
        .unwrap();
    borsh::to_writer(&mut data_writer, &event).unwrap();
    let data_len = data_writer.position() as usize;

    let out_len = base64::encode_config_slice(
        &data_writer.into_inner()[0..data_len],
        base64::STANDARD,
        out_buf,
    );

    let msg_bytes = &out_buf[0..out_len];
    let msg_str = unsafe { std::str::from_utf8_unchecked(&msg_bytes) };

    msg!("landlord-log");
    msg!(msg_str);
}

#[event]
pub struct AssetIssuance {
    pub owner: Pubkey,
    pub asset_id: Pubkey,
    pub owner_pda: Pubkey,
    pub master_edition: Pubkey,
    pub metadata: Pubkey,
    pub iat: i64
}