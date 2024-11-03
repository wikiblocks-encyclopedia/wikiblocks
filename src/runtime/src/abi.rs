use core::marker::PhantomData;

use scale::{Encode, Decode};

use wikiblocks_abi::Call;

use crate::{
  timestamp, coins, articles, votes,
  validator_sets::{self, MembershipProof},
  babe, grandpa, RuntimeCall,
};

impl From<Call> for RuntimeCall {
  fn from(call: Call) -> RuntimeCall {
    match call {
      Call::Timestamp(wikiblocks_abi::timestamp::Call::set { now }) => {
        RuntimeCall::Timestamp(timestamp::Call::set { now })
      }
      Call::Coins(coins) => match coins {
        wikiblocks_abi::coins::Call::transfer { to, amount } => {
          RuntimeCall::Coins(coins::Call::transfer { to: to.into(), amount })
        }
        wikiblocks_abi::coins::Call::burn { amount } => {
          RuntimeCall::Coins(coins::Call::burn { amount })
        }
      },
      Call::ValidatorSets(vs) => match vs {
        wikiblocks_abi::validator_sets::Call::allocate { amount } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::allocate { amount })
        }
        wikiblocks_abi::validator_sets::Call::deallocate { amount } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::deallocate { amount })
        }
        wikiblocks_abi::validator_sets::Call::claim_deallocation { session } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::claim_deallocation { session })
        }
      },
      Call::Articles(articles) => match articles {
        wikiblocks_abi::articles::Call::add_article { script } => {
          RuntimeCall::Articles(articles::Call::add_article { script })
        }
        wikiblocks_abi::articles::Call::add_version { title, script } => {
          RuntimeCall::Articles(articles::Call::add_version { title, script })
        }
      },
      Call::Votes(votes) => match votes {
        wikiblocks_abi::votes::Call::upvote { article } => {
          RuntimeCall::Votes(votes::Call::upvote { article })
        }
      },
      Call::Babe(babe) => match babe {
        wikiblocks_abi::babe::Call::report_equivocation(report) => {
          RuntimeCall::Babe(babe::Call::report_equivocation {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
        wikiblocks_abi::babe::Call::report_equivocation_unsigned(report) => {
          RuntimeCall::Babe(babe::Call::report_equivocation_unsigned {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
      },
      Call::Grandpa(grandpa) => match grandpa {
        wikiblocks_abi::grandpa::Call::report_equivocation(report) => {
          RuntimeCall::Grandpa(grandpa::Call::report_equivocation {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
        wikiblocks_abi::grandpa::Call::report_equivocation_unsigned(report) => {
          RuntimeCall::Grandpa(grandpa::Call::report_equivocation_unsigned {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
      },
    }
  }
}

impl TryInto<Call> for RuntimeCall {
  type Error = ();

  fn try_into(self) -> Result<Call, ()> {
    Ok(match self {
      RuntimeCall::Timestamp(timestamp::Call::set { now }) => {
        Call::Timestamp(wikiblocks_abi::timestamp::Call::set { now })
      }
      RuntimeCall::Coins(call) => Call::Coins(match call {
        coins::Call::transfer { to, amount } => {
          wikiblocks_abi::coins::Call::transfer { to: to.into(), amount }
        }
        coins::Call::burn { amount } => wikiblocks_abi::coins::Call::burn { amount },
        _ => Err(())?,
      }),
      RuntimeCall::ValidatorSets(call) => Call::ValidatorSets(match call {
        validator_sets::Call::allocate { amount } => {
          wikiblocks_abi::validator_sets::Call::allocate { amount }
        }
        validator_sets::Call::deallocate { amount } => {
          wikiblocks_abi::validator_sets::Call::deallocate { amount }
        }
        validator_sets::Call::claim_deallocation { session } => {
          wikiblocks_abi::validator_sets::Call::claim_deallocation { session }
        }
        _ => Err(())?,
      }),
      RuntimeCall::Articles(call) => Call::Articles(match call {
        articles::Call::add_article { script } => {
          wikiblocks_abi::articles::Call::add_article { script }
        }
        articles::Call::add_version { title, script } => {
          wikiblocks_abi::articles::Call::add_version { title, script }
        }
        _ => Err(())?,
      }),
      RuntimeCall::Votes(call) => Call::Votes(match call {
        votes_pallet::Call::upvote { article } => wikiblocks_abi::votes::Call::upvote { article },
        _ => Err(())?,
      }),
      RuntimeCall::Babe(call) => Call::Babe(match call {
        babe::Call::report_equivocation { equivocation_proof, key_owner_proof } => {
          wikiblocks_abi::babe::Call::report_equivocation(
            wikiblocks_abi::babe::ReportEquivocation {
              // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
              equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
              key_owner_proof: key_owner_proof.0.into(),
            },
          )
        }
        babe::Call::report_equivocation_unsigned { equivocation_proof, key_owner_proof } => {
          wikiblocks_abi::babe::Call::report_equivocation_unsigned(
            wikiblocks_abi::babe::ReportEquivocation {
              // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
              equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
              key_owner_proof: key_owner_proof.0.into(),
            },
          )
        }
        _ => Err(())?,
      }),
      RuntimeCall::Grandpa(call) => Call::Grandpa(match call {
        grandpa::Call::report_equivocation { equivocation_proof, key_owner_proof } => {
          wikiblocks_abi::grandpa::Call::report_equivocation(
            wikiblocks_abi::grandpa::ReportEquivocation {
              // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
              equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
              key_owner_proof: key_owner_proof.0.into(),
            },
          )
        }
        grandpa::Call::report_equivocation_unsigned { equivocation_proof, key_owner_proof } => {
          wikiblocks_abi::grandpa::Call::report_equivocation_unsigned(
            wikiblocks_abi::grandpa::ReportEquivocation {
              // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
              equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
              key_owner_proof: key_owner_proof.0.into(),
            },
          )
        }
        _ => Err(())?,
      }),
      _ => Err(())?,
    })
  }
}
