#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

use wikiblocks_primitives::{system_address, WikiblocksAddress};

pub const FEE_ACCOUNT: WikiblocksAddress = system_address(b"Coins-fees");

#[test]
fn address() {
  use sp_runtime::traits::TrailingZeroInput;
  use scale::Decode;
  assert_eq!(
    FEE_ACCOUNT,
    WikiblocksAddress::decode(&mut TrailingZeroInput::new(b"Coins-fees")).unwrap()
  );
}
