use scale::Encode;

use sp_core::sr25519::Public;

use wikiblocks_abi::primitives::SubstrateAmount;
pub use wikiblocks_abi::validator_sets::primitives;
use primitives::Session;

use crate::{TemporalWikiblocks, WikiblocksError};

const PALLET: &str = "ValidatorSets";

pub type ValidatorSetsEvent = wikiblocks_abi::validator_sets::Event;

#[derive(Clone, Copy)]
pub struct WikiblocksValidatorSets<'a>(pub(crate) &'a TemporalWikiblocks<'a>);
impl<'a> WikiblocksValidatorSets<'a> {
  pub async fn new_session_events(&self) -> Result<Vec<ValidatorSetsEvent>, WikiblocksError> {
    self
      .0
      .events(|event| {
        if let wikiblocks_abi::Event::ValidatorSets(event) = event {
          if matches!(event, ValidatorSetsEvent::NewSession { .. }) {
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

  pub async fn participant_removed_events(
    &self,
  ) -> Result<Vec<ValidatorSetsEvent>, WikiblocksError> {
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

  pub async fn set_retired_events(&self) -> Result<Vec<ValidatorSetsEvent>, WikiblocksError> {
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

  pub async fn session(&self) -> Result<Option<Session>, WikiblocksError> {
    self.0.storage(PALLET, "CurrentSession", ()).await
  }

  pub async fn participants(&self) -> Result<Option<Vec<(Public, u64)>>, WikiblocksError> {
    self.0.storage(PALLET, "Participants", ()).await
  }

  pub async fn allocation_per_key_share(&self) -> Result<SubstrateAmount, WikiblocksError> {
    Ok(self.0.storage(PALLET, "AllocationPerKeyShare", ()).await?.unwrap())
  }

  pub async fn total_allocated_stake(&self) -> Result<Option<SubstrateAmount>, WikiblocksError> {
    self.0.storage(PALLET, "TotalAllocatedStake", ()).await
  }

  pub async fn allocation(&self, key: Public) -> Result<Option<SubstrateAmount>, WikiblocksError> {
    self.0.storage(PALLET, "Allocations", (sp_core::hashing::blake2_128(&key.encode()), key)).await
  }

  pub async fn pending_deallocations(
    &self,
    account: Public,
    session: Session,
  ) -> Result<Option<SubstrateAmount>, WikiblocksError> {
    self
      .0
      .storage(
        PALLET,
        "PendingDeallocations",
        (sp_core::hashing::blake2_128(&account.encode()), (account, session)),
      )
      .await
  }

  pub async fn active_network_validators(&self) -> Result<Vec<Public>, WikiblocksError> {
    self.0.runtime_api("WikiblocksRuntimeApi_validators", ()).await
  }

  pub async fn session_begin_block(
    &self,
    session: Session,
  ) -> Result<Option<u64>, WikiblocksError> {
    self.0.storage(PALLET, "SessionBeginBlock", session).await
  }

  pub fn allocate(amount: SubstrateAmount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::ValidatorSets(wikiblocks_abi::validator_sets::Call::allocate { amount })
  }

  pub fn deallocate(amount: SubstrateAmount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::ValidatorSets(wikiblocks_abi::validator_sets::Call::deallocate { amount })
  }
}
