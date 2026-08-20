use anchor_lang::prelude::*;

#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";

#[constant]
pub const HELLO_WORLD_LAMPORTS: u64 = 1;

#[constant]
pub const MAX_COUNT: u64 = 10;

/// GitHub username recorded on-chain by the registration program.
#[constant]
pub const GITHUB_USERNAME: &str = "Anshv784";
