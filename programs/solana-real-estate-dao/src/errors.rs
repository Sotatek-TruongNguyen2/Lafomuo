use anchor_lang::error_code;

#[error_code]
pub enum LafomuoErrors {
    #[msg("Time delay must be greater than zero")]
    TimeDelayMustBeGreaterThanZero
}