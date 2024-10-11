pub use wikiblocks_coins_primitives as primitives;

use wikiblocks_primitives::WikiblocksAddress;

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Call {
  transfer { to: WikiblocksAddress, balance: Balance },
  burn { balance: Balance },
  burn_with_instruction { instruction: OutInstructionWithBalance },
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Event {
  Mint { to: WikiblocksAddress, balance: Balance },
  Burn { from: WikiblocksAddress, balance: Balance },
  BurnWithInstruction { from: WikiblocksAddress, instruction: OutInstructionWithBalance },
  Transfer { from: WikiblocksAddress, to: WikiblocksAddress, balance: Balance },
}
