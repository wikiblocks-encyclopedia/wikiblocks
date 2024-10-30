use scale::Encode;

use wikiblocks_abi::primitives::{ArticleVersion, Script, Title};
pub use wikiblocks_abi::coins::primitives;

use crate::{TemporalWikiblocks, WikiblocksError};

const PALLET: &str = "Articles";

#[derive(Clone, Copy)]
pub struct WikiblocksArticles<'a>(pub(crate) &'a TemporalWikiblocks<'a>);
impl<'a> WikiblocksArticles<'a> {
  pub fn add_article(script: Script) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::Articles(wikiblocks_abi::articles::Call::add_article { script })
  }

  pub fn add_version(title: Title, script: Script) -> wikiblocks_abi::Call {
    wikiblocks_abi::Call::Articles(wikiblocks_abi::articles::Call::add_version { title, script })
  }

  pub async fn article(
    &self,
    title: &Title,
    version: ArticleVersion,
  ) -> Result<Option<Script>, WikiblocksError> {
    self
      .0
      .storage(
        PALLET,
        "Articles",
        (sp_core::hashing::blake2_128(&(title, version).encode()), (title, version)),
      )
      .await
  }

  pub async fn author(
    &self,
    title: &Title,
    version: ArticleVersion,
  ) -> Result<Option<Script>, WikiblocksError> {
    self
      .0
      .storage(
        PALLET,
        "Authors",
        (sp_core::hashing::blake2_128(&(title, version).encode()), (title, version)),
      )
      .await
  }
}
