#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]

extern crate alloc;

pub use wikiblocks_primitives as primitives;

pub mod system;

pub mod timestamp;

pub mod coins;
pub mod validator_sets;

pub mod articles;
pub mod votes;

pub mod babe;
pub mod grandpa;

pub mod tx;

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum Call {
  Timestamp(timestamp::Call),
  Coins(coins::Call),
  ValidatorSets(validator_sets::Call),
  Articles(articles::Call),
  Votes(votes::Call),
  Babe(babe::Call),
  Grandpa(grandpa::Call),
}

// TODO: Remove this
#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum TransactionPaymentEvent {
  TransactionFeePaid { who: wikiblocks_primitives::WikiblocksAddress, actual_fee: u64, tip: u64 },
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum Event {
  System(system::Event),
  Timestamp,
  TransactionPayment(TransactionPaymentEvent),
  Coins(coins::Event),
  ValidatorSets(validator_sets::Event),
  Articles,
  Votes,
  Babe,
  Grandpa(grandpa::Event),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub struct Extra {
  pub era: sp_runtime::generic::Era,
  #[codec(compact)]
  pub nonce: u32,
  #[codec(compact)]
  pub tip: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "std", feature = "serde"), derive(serde::Deserialize))]
pub struct SignedPayloadExtra {
  pub spec_version: u32,
  pub tx_version: u32,
  pub genesis: [u8; 32],
  pub mortality_checkpoint: [u8; 32],
}

pub type Transaction = tx::Transaction<Call, Extra>;
