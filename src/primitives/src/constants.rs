use crate::BlockNumber;

// 1 MB
pub const BLOCK_SIZE: u32 = 1024 * 1024;
// 6 seconds
pub const TARGET_BLOCK_TIME: u64 = 6;

/// Measured in blocks.
pub const MINUTES: BlockNumber = 60 / TARGET_BLOCK_TIME;
pub const HOURS: BlockNumber = 60 * MINUTES;
pub const DAYS: BlockNumber = 24 * HOURS;
pub const WEEKS: BlockNumber = 7 * DAYS;
// Defines a month as 30 days, which is slightly inaccurate
pub const MONTHS: BlockNumber = 30 * DAYS;
// Defines a year as 12 inaccurate months, which is 360 days literally (~1.5% off)
pub const YEARS: BlockNumber = 12 * MONTHS;

// 1000b/1usd rate for data insert into the chain.
// this represent a usd. 1000 mill.
pub const DATA_FEE_RATE: u64 = 1000;

/// Amount of blocks per epoch in the fast-epoch feature that is used in tests.
pub const FAST_EPOCH_DURATION: u64 = MINUTES;

/// REWARD = 10M / BLOCKS_PER_YEAR
pub const REWARD_PER_BLOCK: u64 = (10_000_000 * 10u64.pow(8)) / YEARS;
