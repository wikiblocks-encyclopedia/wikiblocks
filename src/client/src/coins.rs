use scale::Encode;

use wikiblocks_abi::primitives::{SubstrateAmount, WikiblocksAddress};
pub use wikiblocks_abi::coins::primitives;

use crate::{TemporalWikiblocks, WikiblocksError};

const PALLET: &str = "Coins";

pub type CoinsEvent = wikiblocks_abi::coins::Event;

#[derive(Clone, Copy)]
pub struct WikiblocksCoins<'a>(pub(crate) &'a TemporalWikiblocks<'a>);
impl<'a> WikiblocksCoins<'a> {
  pub async fn mint_events(&self) -> Result<Vec<CoinsEvent>, WikiblocksError> {
    self
      .0
      .events(|event| {
        if let wikiblocks_abi::Event::Coins(event) = event {
          if matches!(event, CoinsEvent::Mint { .. }) {
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

  pub async fn supply(&self) -> Result<SubstrateAmount, WikiblocksError> {
    Ok(self.0.storage(PALLET, "Supply", ()).await?.unwrap_or(0))
  }

  pub async fn balance(
    &self,
    address: WikiblocksAddress,
  ) -> Result<SubstrateAmount, WikiblocksError> {
    Ok(
      self
        .0
        .storage(PALLET, "Balances", (sp_core::hashing::blake2_128(&address.encode()), &address.0))
        .await?
        .unwrap_or(0),
    )
  }

  pub fn transfer(to: WikiblocksAddress, amount: SubstrateAmount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::Coins(wikiblocks_abi::coins::Call::transfer { to, amount })
  }

  pub fn burn(amount: SubstrateAmount) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::Coins(wikiblocks_abi::coins::Call::burn { amount })
  }
}
