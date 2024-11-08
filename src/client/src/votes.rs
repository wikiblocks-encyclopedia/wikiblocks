use scale::Encode;

use wikiblocks_abi::primitives::Article;
pub use wikiblocks_abi::coins::primitives;

use crate::{TemporalWikiblocks, WikiblocksError};

const PALLET: &str = "Votes";

#[derive(Clone, Copy)]
pub struct WikiblocksVotes<'a>(pub(crate) &'a TemporalWikiblocks<'a>);
impl<'a> WikiblocksVotes<'a> {
  pub fn upvote(article: Article) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::Votes(wikiblocks_abi::votes::Call::upvote { article })
  }

  pub async fn upvotes(&self, article: &Article) -> Result<Option<u64>, WikiblocksError> {
    self
      .0
      .storage(PALLET, "Upvotes", (sp_core::hashing::blake2_128(&article.encode()), article))
      .await
  }
}
