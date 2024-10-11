use scale::Encode;

use wikiblocks_abi::primitives::{Amount, Balance, Coin, SubstrateAmount, WikiblocksAddress};
pub use wikiblocks_abi::coins::primitives;
use primitives::OutInstructionWithBalance;

use crate::{TemporalSerai, SeraiError};

const PALLET: &str = "Coins";

pub type CoinsEvent = wikiblocks_abi::coins::Event;

#[derive(Clone, Copy)]
pub struct SeraiCoins<'a>(pub(crate) &'a TemporalSerai<'a>);
impl<'a> SeraiCoins<'a> {
  pub async fn mint_events(&self) -> Result<Vec<CoinsEvent>, SeraiError> {
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

  pub async fn coin_supply(&self, coin: Coin) -> Result<SubstrateAmount, SeraiError> {
    self.0.storage(PALLET, "Supply", coin).await
  }

  pub async fn balance(&self, address: WikiblocksAddress) -> Result<SubstrateAmount, SeraiError> {
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
