pub use serai_validator_sets_primitives as primitives;

use serai_primitives::*;
use serai_validator_sets_primitives::*;

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Call {
  allocate { network: NetworkId, amount: Amount },
  deallocate { network: NetworkId, amount: Amount },
  claim_deallocation { network: NetworkId, session: Session },
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub enum Event {
  NewSet {
    set: ValidatorSet,
  },
  ParticipantRemoved {
    set: ValidatorSet,
    removed: SeraiAddress,
  },
  AcceptedHandover {
    set: ValidatorSet,
  },
  SetRetired {
    set: ValidatorSet,
  },
  AllocationIncreased {
    validator: SeraiAddress,
    network: NetworkId,
    amount: Amount,
  },
  AllocationDecreased {
    validator: SeraiAddress,
    network: NetworkId,
    amount: Amount,
    delayed_until: Option<Session>,
  },
  DeallocationClaimed {
    validator: SeraiAddress,
    network: NetworkId,
    session: Session,
  },
}
