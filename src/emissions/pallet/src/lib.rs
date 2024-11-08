#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[allow(
  unreachable_patterns,
  clippy::cast_possible_truncation,
  clippy::no_effect_underscore_binding,
  clippy::empty_docs
)]
#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_system::pallet_prelude::*;
  use frame_support::pallet_prelude::*;

  use sp_std::{vec, vec::Vec};
  use sp_core::sr25519::Public;

  use coins_pallet::{Config as CoinsConfig, Pallet as Coins};
  use validator_sets_pallet::{Pallet as ValidatorSets, Config as ValidatorSetsConfig};

  use validator_sets_primitives::{MAX_KEY_SHARES_PER_SET, Session};
  use wikiblocks_primitives::*;

  #[pallet::config]
  pub trait Config:
    frame_system::Config<AccountId = PublicKey> + ValidatorSetsConfig + CoinsConfig
  {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
  }

  #[pallet::genesis_config]
  #[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
  pub struct GenesisConfig<T: Config> {
    pub participants: Vec<(T::AccountId, SubstrateAmount)>,
  }

  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      GenesisConfig { participants: Default::default() }
    }
  }

  #[pallet::error]
  pub enum Error<T> {
    NetworkHasEconomicSecurity,
    NoValueForCoin,
    InsufficientAllocation,
  }

  #[pallet::event]
  pub enum Event<T: Config> {}

  #[pallet::pallet]
  pub struct Pallet<T>(PhantomData<T>);

  // TODO: Remove this. This should be the sole domain of validator-sets
  #[pallet::storage]
  #[pallet::getter(fn participants)]
  pub(crate) type Participants<T: Config> =
    StorageValue<_, BoundedVec<(Public, u64), ConstU32<{ MAX_KEY_SHARES_PER_SET }>>, OptionQuery>;

  // TODO: Remove this too
  #[pallet::storage]
  #[pallet::getter(fn session)]
  pub type CurrentSession<T: Config> = StorageValue<_, u32, ValueQuery>;

  #[pallet::genesis_build]
  impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
    fn build(&self) {
      Participants::<T>::set(Some(self.participants.clone().try_into().unwrap()));
      CurrentSession::<T>::set(0);
    }
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(_: BlockNumberFor<T>) -> Weight {
      // check if we got a new session
      let mut session_changed = false;
      let session = ValidatorSets::<T>::session().unwrap_or(Session(0));
      if session.0 > Self::session() {
        session_changed = true;
        CurrentSession::<T>::set(session.0);
      }

      // We only want to distribute emissions if session has ended
      if !session_changed {
        return Weight::zero(); // TODO
      }

      // figure out the amount of blocks in the last session
      // Since the session has changed, we're now at least at session 1
      let block_count = ValidatorSets::<T>::session_begin_block(session) -
        ValidatorSets::<T>::session_begin_block(Session(session.0 - 1));

      // get total reward for this epoch
      let reward_this_epoch = block_count * REWARD_PER_BLOCK;

      // distribute validators rewards
      Self::distribute_to_validators(reward_this_epoch);

      // TODO: we have the past session participants here in the emissions pallet so that we can
      // distribute rewards to them in the next session. Ideally we should be able to fetch this
      // information from validator sets pallet.
      Self::update_participants();
      Weight::zero() // TODO
    }
  }

  impl<T: Config> Pallet<T> {
    // Distribute the reward among network's set based on
    // -> (key shares * stake per share) + ((stake % stake per share) / 2)
    fn distribute_to_validators(reward: u64) {
      let stake_per_share = ValidatorSets::<T>::allocation_per_key_share();
      let mut scores = vec![];
      let mut total_score = 0u64;
      for (p, amount) in Self::participants().unwrap() {
        let remainder = amount % stake_per_share;
        let score = amount - (remainder / 2);

        total_score = total_score.saturating_add(score);
        scores.push((p, score));
      }

      // stake the rewards
      let mut total_reward_distributed = 0u64;
      for (i, (p, score)) in scores.iter().enumerate() {
        let p_reward = if i == (scores.len() - 1) {
          reward.saturating_sub(total_reward_distributed)
        } else {
          u64::try_from(
            u128::from(reward).saturating_mul(u128::from(*score)) / u128::from(total_score),
          )
          .unwrap()
        };

        Coins::<T>::mint(*p, p_reward).unwrap();
        ValidatorSets::<T>::distribute_block_rewards(*p, p_reward).unwrap();

        total_reward_distributed = total_reward_distributed.saturating_add(p_reward);
      }
    }

    fn update_participants() {
      let participants = ValidatorSets::<T>::participants_for_latest_decided_set()
        .unwrap()
        .into_iter()
        .map(|(key, _)| (key, ValidatorSets::<T>::allocation(key).unwrap_or(0)))
        .collect::<Vec<_>>();

      Participants::<T>::set(Some(participants.try_into().unwrap()));
    }
  }
}

pub use pallet::*;
