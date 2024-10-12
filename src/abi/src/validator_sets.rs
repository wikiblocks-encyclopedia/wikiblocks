pub use wikiblocks_validator_sets_primitives as primitives;

use wikiblocks_primitives::*;
use primitives::*;

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Call {
  allocate { amount: SubstrateAmount },
  deallocate { amount: SubstrateAmount },
  claim_deallocation { session: Session },
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Event {
  NewSession {
    session: Session,
  },
  ParticipantRemoved {
    session: Session,
    removed: WikiblocksAddress,
  },
  SetRetired {
    session: Session,
  },
  AllocationIncreased {
    validator: WikiblocksAddress,
    amount: SubstrateAmount,
  },
  AllocationDecreased {
    validator: WikiblocksAddress,
    amount: SubstrateAmount,
    delayed_until: Option<Session>,
  },
  DeallocationClaimed {
    validator: WikiblocksAddress,
    session: Session,
  },
}
