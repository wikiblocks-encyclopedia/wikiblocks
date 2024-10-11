use scale::Encode;

use sp_core::sr25519::Public;

use wikiblocks_abi::primitives::Amount;
pub use wikiblocks_abi::validator_sets::primitives;
use primitives::Session;

use crate::{primitives::NetworkId, TemporalSerai, SeraiError};

const PALLET: &str = "ValidatorSets";

pub type ValidatorSetsEvent = wikiblocks_abi::validator_sets::Event;

#[derive(Clone, Copy)]
pub struct SeraiValidatorSets<'a>(pub(crate) &'a TemporalSerai<'a>);
impl<'a> SeraiValidatorSets<'a> {
  pub async fn new_set_events(&self) -> Result<Vec<ValidatorSetsEvent>, SeraiError> {
    self
      .0
      .events(|event| {
        if let wikiblocks_abi::Event::ValidatorSets(event) = event {
          if matches!(event, ValidatorSetsEvent::NewSet { .. }) {
            Some(event.clone())
          } else {
            None
          }
        } else {
          None
        }
      })
      .await
  }

  pub async fn participant_removed_events(&self) -> Result<Vec<ValidatorSetsEvent>, SeraiError> {
    self
      .0
      .events(|event| {
        if let wikiblocks_abi::Event::ValidatorSets(event) = event {
          if matches!(event, ValidatorSetsEvent::ParticipantRemoved { .. }) {
            Some(event.clone())
          } else {
            None
          }
        } else {
          None
        }
      })
      .await
  }

  pub async fn set_retired_events(&self) -> Result<Vec<ValidatorSetsEvent>, SeraiError> {
    self
      .0
      .events(|event| {
        if let wikiblocks_abi::Event::ValidatorSets(event) = event {
          if matches!(event, ValidatorSetsEvent::SetRetired { .. }) {
            Some(event.clone())
          } else {
            None
          }
        } else {
          None
        }
      })
      .await
  }

  pub async fn session(&self, network: NetworkId) -> Result<Option<Session>, SeraiError> {
    self.0.storage(PALLET, "CurrentSession", network).await
  }

  pub async fn participants(
    &self,
    network: NetworkId,
  ) -> Result<Option<Vec<(Public, u64)>>, SeraiError> {
    self.0.storage(PALLET, "Participants", network).await
  }

  pub async fn allocation_per_key_share(
    &self,
    network: NetworkId,
  ) -> Result<Option<Amount>, SeraiError> {
    self.0.storage(PALLET, "AllocationPerKeyShare", network).await
  }

  pub async fn total_allocated_stake(
    &self,
    network: NetworkId,
  ) -> Result<Option<Amount>, SeraiError> {
    self.0.storage(PALLET, "TotalAllocatedStake", network).await
  }

  pub async fn allocation(
    &self,
    network: NetworkId,
    key: Public,
  ) -> Result<Option<Amount>, SeraiError> {
    self
      .0
      .storage(
        PALLET,
        "Allocations",
        (sp_core::hashing::blake2_128(&(network, key).encode()), (network, key)),
      )
      .await
  }

  pub async fn pending_deallocations(
    &self,
    network: NetworkId,
    account: Public,
    session: Session,
  ) -> Result<Option<Amount>, SeraiError> {
    self
      .0
      .storage(
        PALLET,
        "PendingDeallocations",
        (sp_core::hashing::blake2_128(&(network, account).encode()), (network, account, session)),
      )
      .await
  }

  pub async fn active_network_validators(
    &self,
    network: NetworkId,
  ) -> Result<Vec<Public>, SeraiError> {
    self.0.runtime_api("SeraiRuntimeApi_validators", network).await
  }

  pub async fn session_begin_block(
    &self,
    network: NetworkId,
    session: Session,
  ) -> Result<Option<u64>, SeraiError> {
    self.0.storage(PALLET, "SessionBeginBlock", (network, session)).await
  }

  pub fn allocate(network: NetworkId, amount: Amount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::ValidatorSets(wikiblocks_abi::validator_sets::Call::allocate {
      network,
      amount,
    })
  }

  pub fn deallocate(network: NetworkId, amount: Amount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::ValidatorSets(wikiblocks_abi::validator_sets::Call::deallocate {
      network,
      amount,
    })
  }
}
