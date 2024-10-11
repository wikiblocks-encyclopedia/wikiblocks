pub use wikiblocks_coins_primitives as primitives;

use wikiblocks_primitives::{SubstrateAmount, WikiblocksAddress};

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Call {
  transfer { to: WikiblocksAddress, amount: SubstrateAmount },
  burn { amount: SubstrateAmount },
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Event {
  Mint { to: WikiblocksAddress, amount: SubstrateAmount },
  Burn { from: WikiblocksAddress, amount: SubstrateAmount },
  Transfer { from: WikiblocksAddress, to: WikiblocksAddress, amount: SubstrateAmount },
}
