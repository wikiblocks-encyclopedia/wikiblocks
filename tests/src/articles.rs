use crate::{wikiblocks_test, publish_tx};

use wikiblocks_abi::primitives::{insecure_pair_from_name, ArticleVersion, Body, OpCode, Script, Title};
use wikiblocks_client::{Wikiblocks, WikiblocksArticles};

wikiblocks_test!(
  add_article: (|wikiblocks: Wikiblocks| async move {
    test_add_article(wikiblocks).await;
  })
);

#[allow(dead_code)]
async fn test_add_article(wikiblocks: Wikiblocks) {
  let signer = insecure_pair_from_name("Alice");

  // make an article
  let mut script = vec![];
  let title = Title::new("My first article".as_bytes().to_vec()).unwrap();
  script.push(OpCode::Title(title.clone()));
  script.push(OpCode::Add(
    Body::new("this is firs body for the first title".as_bytes().to_vec()).unwrap(),
  ));

  // TODO: add the necessary tip for the tx

  // send the tx
  let script = Script::new(script).unwrap();
  let tx = wikiblocks.sign(&signer, WikiblocksArticles::add_article(script.clone()), 0, 0);
  publish_tx(&wikiblocks, &tx).await;

  // read back
  let result = wikiblocks
    .as_of_latest_finalized_block()
    .await
    .unwrap()
    .articles()
    .article(&title, ArticleVersion(0))
    .await
    .unwrap()
    .unwrap();

  // exclude the title from the script when comparing
  assert_eq!(result.data(), &script.data()[1 ..]);
}
