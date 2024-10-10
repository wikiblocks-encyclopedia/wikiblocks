use core::marker::PhantomData;

use scale::{Encode, Decode};

use serai_abi::Call;

use crate::{
  Vec,
  primitives::{PublicKey, SeraiAddress},
  timestamp, coins,
  validator_sets::{self, MembershipProof},
  babe, grandpa, RuntimeCall,
};

impl From<Call> for RuntimeCall {
  fn from(call: Call) -> RuntimeCall {
    match call {
      Call::Timestamp(serai_abi::timestamp::Call::set { now }) => {
        RuntimeCall::Timestamp(timestamp::Call::set { now })
      }
      Call::Coins(coins) => match coins {
        serai_abi::coins::Call::transfer { to, balance } => {
          RuntimeCall::Coins(coins::Call::transfer { to: to.into(), balance })
        }
        serai_abi::coins::Call::burn { balance } => {
          RuntimeCall::Coins(coins::Call::burn { balance })
        }
        serai_abi::coins::Call::burn_with_instruction { instruction } => {
          RuntimeCall::Coins(coins::Call::burn_with_instruction { instruction })
        }
      },
      Call::ValidatorSets(vs) => match vs {
        serai_abi::validator_sets::Call::set_keys {
          network,
          removed_participants,
          key_pair,
          signature,
        } => RuntimeCall::ValidatorSets(validator_sets::Call::set_keys {
          network,
          removed_participants: <_>::try_from(
            removed_participants.into_iter().map(PublicKey::from).collect::<Vec<_>>(),
          )
          .unwrap(),
          key_pair,
          signature,
        }),
        serai_abi::validator_sets::Call::report_slashes { network, slashes, signature } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::report_slashes {
            network,
            slashes: <_>::try_from(
              slashes
                .into_iter()
                .map(|(addr, slash)| (PublicKey::from(addr), slash))
                .collect::<Vec<_>>(),
            )
            .unwrap(),
            signature,
          })
        }
        serai_abi::validator_sets::Call::allocate { network, amount } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::allocate { network, amount })
        }
        serai_abi::validator_sets::Call::deallocate { network, amount } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::deallocate { network, amount })
        }
        serai_abi::validator_sets::Call::claim_deallocation { network, session } => {
          RuntimeCall::ValidatorSets(validator_sets::Call::claim_deallocation { network, session })
        }
      },
      Call::Babe(babe) => match babe {
        serai_abi::babe::Call::report_equivocation(report) => {
          RuntimeCall::Babe(babe::Call::report_equivocation {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
        serai_abi::babe::Call::report_equivocation_unsigned(report) => {
          RuntimeCall::Babe(babe::Call::report_equivocation_unsigned {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
      },
      Call::Grandpa(grandpa) => match grandpa {
        serai_abi::grandpa::Call::report_equivocation(report) => {
          RuntimeCall::Grandpa(grandpa::Call::report_equivocation {
            // TODO: Find a better way to go from Proof<[u8; 32]> to Proof<H256>
            equivocation_proof: <_>::decode(&mut report.equivocation_proof.encode().as_slice())
              .unwrap(),
            key_owner_proof: MembershipProof(report.key_owner_proof.into(), PhantomData),
          })
        }
        serai_abi::grandpa::Call::report_equivocation_unsigned(report) => {
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
        Call::Timestamp(serai_abi::timestamp::Call::set { now })
      }
      RuntimeCall::Coins(call) => Call::Coins(match call {
        coins::Call::transfer { to, balance } => {
          serai_abi::coins::Call::transfer { to: to.into(), balance }
        }
        coins::Call::burn { balance } => serai_abi::coins::Call::burn { balance },
        coins::Call::burn_with_instruction { instruction } => {
          serai_abi::coins::Call::burn_with_instruction { instruction }
        }
        _ => Err(())?,
      }),
      RuntimeCall::ValidatorSets(call) => Call::ValidatorSets(match call {
        validator_sets::Call::set_keys { network, removed_participants, key_pair, signature } => {
          serai_abi::validator_sets::Call::set_keys {
            network,
            removed_participants: <_>::try_from(
              removed_participants.into_iter().map(SeraiAddress::from).collect::<Vec<_>>(),
            )
            .unwrap(),
            key_pair,
            signature,
          }
        }
        validator_sets::Call::report_slashes { network, slashes, signature } => {
          serai_abi::validator_sets::Call::report_slashes {
            network,
            slashes: <_>::try_from(
              slashes
                .into_iter()
                .map(|(addr, slash)| (SeraiAddress::from(addr), slash))
                .collect::<Vec<_>>(),
            )
            .unwrap(),
            signature,
          }
        }
        validator_sets::Call::allocate { network, amount } => {
          serai_abi::validator_sets::Call::allocate { network, amount }
        }
        validator_sets::Call::deallocate { network, amount } => {
          serai_abi::validator_sets::Call::deallocate { network, amount }
        }
        validator_sets::Call::claim_deallocation { network, session } => {
          serai_abi::validator_sets::Call::claim_deallocation { network, session }
        }
        _ => Err(())?,
      }),
      RuntimeCall::Babe(call) => Call::Babe(match call {
        babe::Call::report_equivocation { equivocation_proof, key_owner_proof } => {
          serai_abi::babe::Call::report_equivocation(serai_abi::babe::ReportEquivocation {
            // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
            equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
            key_owner_proof: key_owner_proof.0.into(),
          })
        }
        babe::Call::report_equivocation_unsigned { equivocation_proof, key_owner_proof } => {
          serai_abi::babe::Call::report_equivocation_unsigned(serai_abi::babe::ReportEquivocation {
            // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
            equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
            key_owner_proof: key_owner_proof.0.into(),
          })
        }
        _ => Err(())?,
      }),
      RuntimeCall::Grandpa(call) => Call::Grandpa(match call {
        grandpa::Call::report_equivocation { equivocation_proof, key_owner_proof } => {
          serai_abi::grandpa::Call::report_equivocation(serai_abi::grandpa::ReportEquivocation {
            // TODO: Find a better way to go from Proof<H256> to Proof<[u8; 32]>
            equivocation_proof: <_>::decode(&mut equivocation_proof.encode().as_slice()).unwrap(),
            key_owner_proof: key_owner_proof.0.into(),
          })
        }
        grandpa::Call::report_equivocation_unsigned { equivocation_proof, key_owner_proof } => {
          serai_abi::grandpa::Call::report_equivocation_unsigned(
            serai_abi::grandpa::ReportEquivocation {
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
