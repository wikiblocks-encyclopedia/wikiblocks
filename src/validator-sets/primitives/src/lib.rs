#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use zeroize::Zeroize;

use scale::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "borsh")]
use borsh::{BorshSerialize, BorshDeserialize};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use sp_core::sr25519::Public;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

/// The maximum amount of key shares per set.
pub const MAX_KEY_SHARES_PER_SET: u32 = 150;

/// The type used to identify a specific session of validators.
#[derive(
  Clone, Copy, PartialEq, Eq, Hash, Default, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Zeroize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Session(pub u32);

/// For a set of validators whose key shares may exceed the maximum, reduce until they equal the
/// maximum.
///
/// Reduction occurs by reducing each validator in a reverse round-robin.
pub fn amortize_excess_key_shares(validators: &mut [(Public, u64)]) {
  let total_key_shares = validators.iter().map(|(_, shares)| shares).sum::<u64>();
  for i in 0 .. usize::try_from(total_key_shares.saturating_sub(u64::from(MAX_KEY_SHARES_PER_SET)))
    .unwrap()
  {
    validators[validators.len() - ((i % validators.len()) + 1)].1 -= 1;
  }
}

/// Returns the post-amortization key shares for the top validator.
///
/// Panics when `validators == 0`.
pub fn post_amortization_key_shares_for_top_validator(
  validators: usize,
  top: u64,
  key_shares: u64,
) -> u64 {
  top -
    (key_shares.saturating_sub(MAX_KEY_SHARES_PER_SET.into()) /
      u64::try_from(validators).unwrap())
}
