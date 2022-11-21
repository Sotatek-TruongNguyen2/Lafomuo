// 2. Add some useful constants for sizing properties (measured in bytes).
pub const DISCRIMINATOR_LENGTH: usize = 8;
pub const PUBLIC_KEY_LENGTH: usize = 32;
pub const U128_LENGTH: usize = 16;
pub const BOOL_LENGTH: usize = 1;
pub const VEC_LENGTH_PREFIX: usize = 4;
pub const MAX_TOPIC_LENGTH: usize = 10 * 4; // 10 chars max.