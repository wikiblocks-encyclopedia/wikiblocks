use frame_support::dispatch::{DispatchInfo, DispatchError};

use wikiblocks_primitives::WikiblocksAddress;

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum Event {
  ExtrinsicSuccess { dispatch_info: DispatchInfo },
  ExtrinsicFailed { dispatch_error: DispatchError, dispatch_info: DispatchInfo },
  CodeUpdated,
  NewAccount { account: WikiblocksAddress },
  KilledAccount { account: WikiblocksAddress },
  Remarked { sender: WikiblocksAddress, hash: [u8; 32] },
}
