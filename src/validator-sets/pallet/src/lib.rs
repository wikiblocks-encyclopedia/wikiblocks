#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use scale::{Encode, Decode};
use scale_info::TypeInfo;

use sp_std::{vec, vec::Vec};
use sp_core::sr25519::Public;
use sp_session::{ShouldEndSession, GetSessionNumber, GetValidatorCount};
use sp_runtime::{KeyTypeId, ConsensusEngineId, traits::IsMember};
use sp_staking::offence::{ReportOffence, Offence, OffenceError};

use frame_system::{pallet_prelude::*, RawOrigin};
use frame_support::{
  pallet_prelude::*,
  sp_runtime::SaturatedConversion,
  traits::{DisabledValidators, KeyOwnerProofSystem, FindAuthor},
  BoundedVec, WeakBoundedVec, StoragePrefixedMap,
};

use wikiblocks_primitives::*;
pub use validator_sets_primitives as primitives;
use primitives::*;

use coins_pallet::Pallet as Coins;

use pallet_babe::{
  Pallet as Babe, AuthorityId as BabeAuthorityId, EquivocationOffence as BabeEquivocationOffence,
};
use pallet_grandpa::{
  Pallet as Grandpa, AuthorityId as GrandpaAuthorityId,
  EquivocationOffence as GrandpaEquivocationOffence,
};

#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone)]
pub struct MembershipProof<T: pallet::Config>(pub Public, pub PhantomData<T>);
impl<T: pallet::Config> GetSessionNumber for MembershipProof<T> {
  fn session(&self) -> u32 {
    let current = Pallet::<T>::session().unwrap().0;
    if Babe::<T>::is_member(&BabeAuthorityId::from(self.0)) {
      current
    } else {
      // if it isn't in the current session, it should have been in the previous one.
      current - 1
    }
  }
}
impl<T: pallet::Config> GetValidatorCount for MembershipProof<T> {
  // We only implement and this interface to satisfy trait requirements
  // Although this might return the wrong count if the offender was in the previous set, we don't
  // rely on it and Substrate only relies on it to offer economic calculations we also don't rely
  // on
  fn validator_count(&self) -> u32 {
    u32::try_from(Babe::<T>::authorities().len()).unwrap()
  }
}

#[allow(
  deprecated,
  unreachable_patterns,
  clippy::let_unit_value,
  clippy::cast_possible_truncation,
  clippy::ignored_unit_patterns
)] // TODO
#[frame_support::pallet]
pub mod pallet {
  use super::*;

  #[pallet::config]
  pub trait Config:
    frame_system::Config<AccountId = Public>
    + coins_pallet::Config
    + pallet_babe::Config
    + pallet_grandpa::Config
    + TypeInfo
  {
    type RuntimeEvent: IsType<<Self as frame_system::Config>::RuntimeEvent> + From<Event<Self>>;

    type ShouldEndSession: ShouldEndSession<BlockNumberFor<Self>>;
  }

  #[pallet::genesis_config]
  #[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
  pub struct GenesisConfig<T: Config> {
    /// Networks to spawn Wikiblocks with, and the stake requirement per key share.
    ///
    /// Every participant at genesis will automatically be assumed to have this much stake.
    /// This stake cannot be withdrawn however as there's no actual stake behind it.
    pub key_share_amount: SubstrateAmount,
    /// List of participants to place in the initial validator sets.
    pub participants: Vec<(T::AccountId, SubstrateAmount)>,
  }

  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      GenesisConfig { key_share_amount: Default::default(), participants: Default::default() }
    }
  }

  #[pallet::pallet]
  pub struct Pallet<T>(PhantomData<T>);

  /// The current session for a network.
  // Uses Identity for the lookup to avoid a hash of a severely limited fixed key-space.
  #[pallet::storage]
  #[pallet::getter(fn session)]
  pub type CurrentSession<T: Config> = StorageValue<_, Session, OptionQuery>;

  /// The allocation required per key share.
  // Uses Identity for the lookup to avoid a hash of a severely limited fixed key-space.
  #[pallet::storage]
  #[pallet::getter(fn allocation_per_key_share)]
  pub type AllocationPerKeyShare<T: Config> = StorageValue<_, SubstrateAmount, ValueQuery>;

  /// The validators selected to be in-set (and their key shares), regardless of if removed.
  ///
  /// This method allows iterating over all validators and their stake.
  #[pallet::storage]
  #[pallet::getter(fn participants_for_latest_decided_set)]
  pub(crate) type Participants<T: Config> =
    StorageValue<_, BoundedVec<(Public, u64), ConstU32<{ MAX_KEY_SHARES_PER_SET }>>, OptionQuery>;

  /// The validators selected to be in-set, regardless of if removed.
  ///
  /// This method allows quickly checking for presence in-set and looking up a validator's key
  /// shares.
  // Uses Identity for NetworkId to avoid a hash of a severely limited fixed key-space.
  #[pallet::storage]
  pub(crate) type InSet<T: Config> = StorageMap<_, Blake2_128Concat, Public, u64, OptionQuery>;

  impl<T: Config> Pallet<T> {
    // This exists as InSet, for Wikiblocks, is the validators set for the next session, *not* the
    // current set's validators
    fn in_active_set(account: Public) -> bool {
      // TODO: is_member is internally O(n). Update Babe to use an O(1) storage lookup?
      Babe::<T>::is_member(&BabeAuthorityId::from(account))
    }

    /// Returns true if the account has been definitively included in an active or upcoming set.
    ///
    /// This will still include participants which were removed from the DKG.
    pub fn in_set(account: Public) -> bool {
      if InSet::<T>::contains_key(account) {
        true
      } else {
        Self::in_active_set(account)
      }
    }
  }

  /// The total stake allocated to this network by the active set of validators.
  #[pallet::storage]
  #[pallet::getter(fn total_allocated_stake)]
  pub type TotalAllocatedStake<T: Config> = StorageValue<_, SubstrateAmount, OptionQuery>;

  /// The current amount allocated to a validator set by a validator.
  #[pallet::storage]
  #[pallet::getter(fn allocation)]
  pub type Allocations<T: Config> =
    StorageMap<_, Blake2_128Concat, Public, SubstrateAmount, OptionQuery>;

  /// A sorted view of the current allocations premised on the underlying DB itself being sorted.
  /*
    This uses Identity so we can take advantage of the DB's lexicographic ordering to iterate over
    the key space from highest-to-lowest allocated.

    This does remove the protection using a hash algorithm here offers against spam attacks (by
    flooding the DB with layers, increasing lookup time and merkle proof sizes, not that we use
    merkle proofs as Polkadot does).

    Since amounts are represented with just 8 bytes, only 16 nibbles are presents. This caps the
    potential depth caused by spam at 16 layers (as the underlying DB operates on nibbles).

    While there is an entire 32-byte public key after this, a Blake hash of the key is inserted
    after the amount to prevent the key from also being used to cause layer spam.

    There's also a minimum stake requirement, which further reduces the potential for spam.
  */
  #[pallet::storage]
  type SortedAllocations<T: Config> =
    StorageMap<_, Identity, ([u8; 8], [u8; 16], Public), (), OptionQuery>;
  impl<T: Config> Pallet<T> {
    #[inline]
    fn sorted_allocation_key(key: Public, amount: SubstrateAmount) -> ([u8; 8], [u8; 16], Public) {
      let amount = reverse_lexicographic_order(amount.to_be_bytes());
      let hash = sp_io::hashing::blake2_128(&(amount, key).encode());
      (amount, hash, key)
    }
    fn recover_amount_from_sorted_allocation_key(key: &[u8]) -> SubstrateAmount {
      let distance_from_end = 8 + 16 + 32;
      let start_pos = key.len() - distance_from_end;
      let mut raw: [u8; 8] = key[start_pos .. (start_pos + 8)].try_into().unwrap();
      for byte in &mut raw {
        *byte = !*byte;
      }
      u64::from_be_bytes(raw)
    }
    fn recover_key_from_sorted_allocation_key(key: &[u8]) -> Public {
      Public(key[(key.len() - 32) ..].try_into().unwrap())
    }

    // Returns if this validator already had an allocation set.
    fn set_allocation(key: Public, amount: SubstrateAmount) -> bool {
      let prior = Allocations::<T>::take(key);
      if let Some(amount) = prior {
        SortedAllocations::<T>::remove(Self::sorted_allocation_key(key, amount));
      }
      if amount != 0 {
        Allocations::<T>::set(key, Some(amount));
        SortedAllocations::<T>::set(Self::sorted_allocation_key(key, amount), Some(()));
      }
      prior.is_some()
    }
  }

  // Doesn't use PrefixIterator as we need to yield the keys *and* values
  // PrefixIterator only yields the values
  struct SortedAllocationsIter<T: Config> {
    _t: PhantomData<T>,
    prefix: Vec<u8>,
    last: Vec<u8>,
    allocation_per_key_share: SubstrateAmount,
  }
  impl<T: Config> SortedAllocationsIter<T> {
    fn new() -> Self {
      let prefix = SortedAllocations::<T>::final_prefix().to_vec();
      Self {
        _t: PhantomData,
        prefix: prefix.clone(),
        last: prefix,
        allocation_per_key_share: Pallet::<T>::allocation_per_key_share(),
      }
    }
  }
  impl<T: Config> Iterator for SortedAllocationsIter<T> {
    type Item = (Public, SubstrateAmount);
    fn next(&mut self) -> Option<Self::Item> {
      let next = sp_io::storage::next_key(&self.last)?;
      if !next.starts_with(&self.prefix) {
        None?;
      }
      let key = Pallet::<T>::recover_key_from_sorted_allocation_key(&next);
      let amount = Pallet::<T>::recover_amount_from_sorted_allocation_key(&next);

      // We may have validators present, with less than the minimum allocation, due to block
      // rewards
      if amount < self.allocation_per_key_share {
        None?;
      }

      self.last = next;
      Some((key, amount))
    }
  }

  /// Pending deallocations, keyed by the Session they become unlocked on.
  #[pallet::storage]
  type PendingDeallocations<T: Config> =
    StorageDoubleMap<_, Blake2_128Concat, Public, Identity, Session, SubstrateAmount, OptionQuery>;

  /// Disabled validators.
  #[pallet::storage]
  pub type DisabledIndices<T: Config> = StorageMap<_, Identity, u32, Public, OptionQuery>;

  /// Mapping from session to its starting block number.
  #[pallet::storage]
  #[pallet::getter(fn session_begin_block)]
  pub type SessionBeginBlock<T: Config> = StorageMap<_, Identity, Session, u64, ValueQuery>;

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    NewSession {
      session: Session,
    },
    ParticipantRemoved {
      session: Session,
      removed: T::AccountId,
    },
    SetRetired {
      session: Session,
    },
    AllocationIncreased {
      validator: T::AccountId,
      amount: SubstrateAmount,
    },
    AllocationDecreased {
      validator: T::AccountId,
      amount: SubstrateAmount,
      delayed_until: Option<Session>,
    },
    DeallocationClaimed {
      validator: T::AccountId,
      session: Session,
    },
  }

  impl<T: Config> Pallet<T> {
    pub fn new_session() {
      // TODO: prevent new set if it doesn't have enough stake for economic security.

      // Update CurrentSession
      let session = {
        let new_session =
          CurrentSession::<T>::get().map_or(Session(0), |session| Session(session.0 + 1));
        CurrentSession::<T>::set(Some(new_session));
        new_session
      };

      // Clear the current InSet
      assert_eq!(InSet::<T>::clear(MAX_KEY_SHARES_PER_SET, None).maybe_cursor, None);

      let mut total_allocated_stake = 0;
      let mut participants = vec![];
      {
        let mut iter = SortedAllocationsIter::<T>::new();
        let mut key_shares = 0;
        while key_shares < u64::from(MAX_KEY_SHARES_PER_SET) {
          let Some((key, amount)) = iter.next() else { break };

          let these_key_shares =
            (amount / Self::allocation_per_key_share()).min(u64::from(MAX_KEY_SHARES_PER_SET));
          participants.push((key, these_key_shares));

          total_allocated_stake += amount;
          key_shares += these_key_shares;
        }
        amortize_excess_key_shares(&mut participants);
      }

      for (key, shares) in &participants {
        InSet::<T>::set(key, Some(*shares));
      }

      Pallet::<T>::deposit_event(Event::NewSession { session });

      Participants::<T>::set(Some(participants.try_into().unwrap()));
      TotalAllocatedStake::<T>::set(Some(total_allocated_stake));

      SessionBeginBlock::<T>::set(
        session,
        <frame_system::Pallet<T>>::block_number().saturated_into::<u64>(),
      );
    }
  }

  #[pallet::error]
  pub enum Error<T> {
    /// Validator Set doesn't exist.
    NonExistentValidatorSet,
    /// Not enough allocation to obtain a key share in the set.
    InsufficientAllocation,
    /// Trying to deallocate more than allocated.
    NotEnoughAllocated,
    /// Allocation would cause the validator set to no longer achieve fault tolerance.
    AllocationWouldRemoveFaultTolerance,
    /// Allocation would cause the validator set to never be able to achieve fault tolerance.
    AllocationWouldPreventFaultTolerance,
    /// Deallocation would remove the participant from the set, despite the validator not
    /// specifying so.
    DeallocationWouldRemoveParticipant,
    /// Deallocation would cause the validator set to no longer achieve fault tolerance.
    DeallocationWouldRemoveFaultTolerance,
    /// Deallocation to be claimed doesn't exist.
    NonExistentDeallocation,
    /// Validator Set already generated keys.
    AlreadyGeneratedKeys,
    /// An invalid MuSig signature was provided.
    BadSignature,
    /// Validator wasn't registered or active.
    NonExistentValidator,
    /// Deallocation would take the stake below what is required.
    DeallocationWouldRemoveEconomicSecurity,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
      if T::ShouldEndSession::should_end_session(n) {
        Self::rotate_session();
        // TODO: set the proper weights
        T::BlockWeights::get().max_block
      } else {
        Weight::zero()
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
    fn build(&self) {
      AllocationPerKeyShare::<T>::set(self.key_share_amount);
      for (participant, stake) in self.participants.clone() {
        if Pallet::<T>::set_allocation(participant, stake) {
          panic!("participants contained duplicates");
        }
      }
      Pallet::<T>::new_session();
    }
  }

  impl<T: Config> Pallet<T> {
    fn account() -> T::AccountId {
      system_address(b"ValidatorSets").into()
    }

    // is_bft returns if the network is able to survive any single node becoming byzantine.
    fn is_bft() -> bool {
      let mut validators_len = 0;
      let mut top = None;
      let mut key_shares = 0;
      for (_, amount) in SortedAllocationsIter::<T>::new() {
        validators_len += 1;

        key_shares += amount / AllocationPerKeyShare::<T>::get();
        if top.is_none() {
          top = Some(key_shares);
        }

        if key_shares > u64::from(MAX_KEY_SHARES_PER_SET) {
          break;
        }
      }

      let Some(top) = top else { return false };

      // key_shares may be over MAX_KEY_SHARES_PER_SET, which will cause a round robin reduction of
      // each validator's key shares until their sum is MAX_KEY_SHARES_PER_SET
      // post_amortization_key_shares_for_top_validator yields what the top validator's key shares
      // would be after such a reduction, letting us evaluate this correctly
      let top = post_amortization_key_shares_for_top_validator(validators_len, top, key_shares);
      (top * 3) < key_shares.min(MAX_KEY_SHARES_PER_SET.into())
    }

    fn increase_allocation(
      account: T::AccountId,
      amount: SubstrateAmount,
      block_reward: bool,
    ) -> DispatchResult {
      let old_allocation = Self::allocation(account).unwrap_or(0);
      let new_allocation = old_allocation + amount;
      let allocation_per_key_share = Self::allocation_per_key_share();
      // If this is a block reward, we always allow it to be allocated
      if (new_allocation < allocation_per_key_share) && (!block_reward) {
        Err(Error::<T>::InsufficientAllocation)?;
      }

      let increased_key_shares =
        (old_allocation / allocation_per_key_share) < (new_allocation / allocation_per_key_share);

      // Check if the net exhibited the ability to handle any single node becoming byzantine
      let mut was_bft = None;
      if increased_key_shares {
        was_bft = Some(Self::is_bft());
      }

      // Increase the allocation now
      Self::set_allocation(account, new_allocation);
      Self::deposit_event(Event::AllocationIncreased { validator: account, amount });

      // Error if the net no longer can handle any single node becoming byzantine
      if let Some(was_bft) = was_bft {
        if was_bft && (!Self::is_bft()) {
          Err(Error::<T>::AllocationWouldRemoveFaultTolerance)?;
        }
      }

      // The above is_bft calls are only used to check a BFT net doesn't become non-BFT
      // Check here if this call would prevent a non-BFT net from *ever* becoming BFT
      if (new_allocation / allocation_per_key_share) >= (MAX_KEY_SHARES_PER_SET / 3).into() {
        Err(Error::<T>::AllocationWouldPreventFaultTolerance)?;
      }

      // If they're in the current set, and the current set has completed its handover (so its
      // currently being tracked by TotalAllocatedStake), update the TotalAllocatedStake
      if InSet::<T>::contains_key(account) {
        TotalAllocatedStake::<T>::set(Some(TotalAllocatedStake::<T>::get().unwrap_or(0) + amount));
      }

      Ok(())
    }

    fn session_to_unlock_on_for_current_set() -> Option<Session> {
      let mut to_unlock_on = Self::session()?;
      // Move to the next session, as deallocating currently in-use stake is obviously invalid
      to_unlock_on.0 += 1;
      // Since the next set will already have been decided, we can only deallocate one
      // session later
      to_unlock_on.0 += 1;
      // Increase the session by one, creating a cooldown period
      to_unlock_on.0 += 1;
      Some(to_unlock_on)
    }

    /// Decreases a validator's allocation to a set.
    ///
    /// Errors if the capacity provided by this allocation is in use.
    ///
    /// Errors if a partial decrease of allocation which puts the remaining allocation below the
    /// minimum requirement.
    ///
    /// The capacity prior provided by the allocation is immediately removed, in order to ensure it
    /// doesn't become used (preventing deallocation).
    ///
    /// Returns if the amount is immediately eligible for deallocation.
    fn decrease_allocation(
      account: T::AccountId,
      amount: SubstrateAmount,
    ) -> Result<bool, DispatchError> {
      let old_allocation = Self::allocation(account).ok_or(Error::<T>::NonExistentValidator)?;
      let new_allocation =
        old_allocation.checked_sub(amount).ok_or(Error::<T>::NotEnoughAllocated)?;

      // If we're not removing the entire allocation, yet the allocation is no longer at or above
      // the threshold for a key share, error
      let allocation_per_key_share = Self::allocation_per_key_share();
      if (new_allocation != 0) && (new_allocation < allocation_per_key_share) {
        Err(Error::<T>::DeallocationWouldRemoveParticipant)?;
      }

      let decreased_key_shares =
        (old_allocation / allocation_per_key_share) > (new_allocation / allocation_per_key_share);

      // If this decreases the validator's key shares, error if the new set is unable to handle
      // byzantine faults
      let mut was_bft = None;
      if decreased_key_shares {
        was_bft = Some(Self::is_bft());
      }

      // Decrease the allocation now
      // Since we don't also update TotalAllocatedStake here, TotalAllocatedStake may be greater
      // than the sum of all allocations, according to the Allocations StorageMap
      // This is intentional as this allocation has only been queued for deallocation at this time
      Self::set_allocation(account, new_allocation);

      if let Some(was_bft) = was_bft {
        if was_bft && (!Self::is_bft()) {
          Err(Error::<T>::DeallocationWouldRemoveFaultTolerance)?;
        }
      }

      // If we're not in-set, allow immediate deallocation
      if !Self::in_set(account) {
        Self::deposit_event(Event::AllocationDecreased {
          validator: account,
          amount,
          delayed_until: None,
        });
        return Ok(true);
      }

      // Set it to PendingDeallocations, letting it be released upon a future session
      // This unwrap should be fine as this account is active, meaning a session has occurred
      let to_unlock_on = Self::session_to_unlock_on_for_current_set().unwrap();
      let existing = PendingDeallocations::<T>::get(account, to_unlock_on).unwrap_or(0);
      PendingDeallocations::<T>::set(account, to_unlock_on, Some(existing + amount));

      Self::deposit_event(Event::AllocationDecreased {
        validator: account,
        amount,
        delayed_until: Some(to_unlock_on),
      });

      Ok(false)
    }

    fn set_total_allocated_stake() {
      let participants = Participants::<T>::get()
        .expect("setting TotalAllocatedStake for a network without participants");
      let total_stake = participants
        .iter()
        .fold(0, |acc, (addr, _)| acc + Allocations::<T>::get(addr).unwrap_or(0));
      TotalAllocatedStake::<T>::set(Some(total_stake));
    }

    // TODO: This is called retire_set, yet just starts retiring the set
    // Update the nomenclature within this function
    pub fn retire_set(session: Session) {
      // emit the event for wikiblocks network
      Self::deposit_event(Event::SetRetired { session });

      // Update the total allocated stake to be for the current set
      Self::set_total_allocated_stake();
    }

    /// Take the amount deallocatable.
    ///
    /// `session` refers to the Session the stake becomes deallocatable on.
    fn take_deallocatable_amount(session: Session, key: Public) -> Option<SubstrateAmount> {
      PendingDeallocations::<T>::take(key, session)
    }

    fn rotate_session() {
      // next wikiblocks validators that is in the queue.
      let now_validators =
        Participants::<T>::get().expect("no Wikiblocks participants upon rotate_session");
      let prior_wikiblocks_session = Self::session().unwrap();

      // TODO: T::SessionHandler::on_before_session_ending() was here.
      // end the current wikiblocks session.
      Self::retire_set(prior_wikiblocks_session);

      // make a new session and get the next validator set.
      Self::new_session();

      // Update Babe and Grandpa
      let session = prior_wikiblocks_session.0 + 1;
      let next_validators = Participants::<T>::get().unwrap();
      Babe::<T>::enact_epoch_change(
        WeakBoundedVec::force_from(
          now_validators.iter().copied().map(|(id, w)| (BabeAuthorityId::from(id), w)).collect(),
          None,
        ),
        WeakBoundedVec::force_from(
          next_validators.iter().copied().map(|(id, w)| (BabeAuthorityId::from(id), w)).collect(),
          None,
        ),
        Some(session),
      );
      Grandpa::<T>::new_session(
        true,
        session,
        now_validators.into_iter().map(|(id, w)| (GrandpaAuthorityId::from(id), w)).collect(),
      );

      // Clear DisabledIndices, only preserving keys still present in the new session
      // First drain so we don't mutate as we iterate
      let mut disabled = vec![];
      for (_, validator) in DisabledIndices::<T>::drain() {
        disabled.push(validator);
      }
      for disabled in disabled {
        Self::disable_validator(disabled);
      }
    }

    pub fn distribute_block_rewards(
      account: T::AccountId,
      amount: SubstrateAmount,
    ) -> DispatchResult {
      // TODO: Should this call be part of the `increase_allocation` since we have to have it
      // before each call to it?
      Coins::<T>::transfer_internal(account, Self::account(), amount)?;
      Self::increase_allocation(account, amount, true)
    }

    fn can_slash_validator(validator: Public) -> bool {
      // Checks if they're active or actively deallocating (letting us still slash them)
      // We could check if they're upcoming/still allocating, yet that'd mean the equivocation is
      // invalid (as they aren't actively signing anything) or severely dated
      // It's not an edge case worth being comprehensive to due to the complexity of being so
      Babe::<T>::is_member(&BabeAuthorityId::from(validator)) ||
        PendingDeallocations::<T>::iter_prefix(validator).next().is_some()
    }

    fn slash_validator(validator: Public) {
      let mut allocation = Self::allocation(validator).unwrap_or(0);
      // reduce the current allocation to 0.
      Self::set_allocation(validator, 0);

      // Take the pending deallocation from the current session
      allocation += PendingDeallocations::<T>::take(
        validator,
        Self::session_to_unlock_on_for_current_set().unwrap(),
      )
      .unwrap_or(0);

      // Reduce the TotalAllocatedStake for the network, if in set
      // TotalAllocatedStake is the sum of allocations and pending deallocations from the current
      // session, since pending deallocations can still be slashed and therefore still contribute
      // to economic security, hence the allocation calculations above being above and the ones
      // below being below
      if InSet::<T>::contains_key(validator) {
        let current_staked = Self::total_allocated_stake().unwrap();
        TotalAllocatedStake::<T>::set(Some(current_staked - allocation));
      }

      // Clear any other pending deallocations.
      for (_, pending) in PendingDeallocations::<T>::drain_prefix(validator) {
        allocation += pending;
      }

      // burn the allocation from the stake account
      Coins::<T>::burn(RawOrigin::Signed(Self::account()).into(), allocation).unwrap();
    }

    /// Disable a validator, preventing them from further authoring blocks.
    ///
    /// Returns true if the validator-to-disable was actually a validator.
    /// Returns false if they weren't.
    fn disable_validator(validator: Public) -> bool {
      if let Some(index) =
        Babe::<T>::authorities().into_iter().position(|(id, _)| id.into_inner() == validator)
      {
        DisabledIndices::<T>::set(u32::try_from(index).unwrap(), Some(validator));

        let session = Self::session().unwrap();
        Self::deposit_event(Event::ParticipantRemoved { session, removed: validator });

        true
      } else {
        false
      }
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight(0)] // TODO
    pub fn allocate(origin: OriginFor<T>, amount: SubstrateAmount) -> DispatchResult {
      let validator = ensure_signed(origin)?;
      Coins::<T>::transfer_internal(validator, Self::account(), amount)?;
      Self::increase_allocation(validator, amount, false)
    }

    #[pallet::call_index(1)]
    #[pallet::weight(0)] // TODO
    pub fn deallocate(origin: OriginFor<T>, amount: SubstrateAmount) -> DispatchResult {
      let account = ensure_signed(origin)?;

      let can_immediately_deallocate = Self::decrease_allocation(account, amount)?;
      if can_immediately_deallocate {
        Coins::<T>::transfer_internal(Self::account(), account, amount)?;
      }

      Ok(())
    }

    #[pallet::call_index(2)]
    #[pallet::weight((0, DispatchClass::Operational))] // TODO
    pub fn claim_deallocation(origin: OriginFor<T>, session: Session) -> DispatchResult {
      let account = ensure_signed(origin)?;
      let Some(amount) = Self::take_deallocatable_amount(session, account) else {
        Err(Error::<T>::NonExistentDeallocation)?
      };
      Coins::<T>::transfer_internal(Self::account(), account, amount)?;
      Self::deposit_event(Event::DeallocationClaimed { validator: account, session });
      Ok(())
    }
  }

  #[rustfmt::skip]
  impl<T: Config, V: Into<Public> + From<Public>> KeyOwnerProofSystem<(KeyTypeId, V)> for Pallet<T> {
    type Proof = MembershipProof<T>;
    type IdentificationTuple = Public;

    fn prove(key: (KeyTypeId, V)) -> Option<Self::Proof> {
      Some(MembershipProof(key.1.into(), PhantomData))
    }

    fn check_proof(key: (KeyTypeId, V), proof: Self::Proof) -> Option<Self::IdentificationTuple> {
      let validator = key.1.into();

      // check the offender and the proof offender are the same.
      if validator != proof.0 {
        return None;
      }

      // check validator is valid
      if !Self::can_slash_validator(validator) {
        return None;
      }

      Some(validator)
    }
  }

  impl<T: Config> ReportOffence<Public, Public, BabeEquivocationOffence<Public>> for Pallet<T> {
    /// Report an `offence` and reward given `reporters`.
    fn report_offence(
      _: Vec<Public>,
      offence: BabeEquivocationOffence<Public>,
    ) -> Result<(), OffenceError> {
      // slash the offender
      let offender = offence.offender;
      Self::slash_validator(offender);

      // disable it
      Self::disable_validator(offender);

      Ok(())
    }

    fn is_known_offence(
      offenders: &[Public],
      _: &<BabeEquivocationOffence<Public> as Offence<Public>>::TimeSlot,
    ) -> bool {
      for offender in offenders {
        // It's not a known offence if we can still slash them
        if Self::can_slash_validator(*offender) {
          return false;
        }
      }
      true
    }
  }

  impl<T: Config> ReportOffence<Public, Public, GrandpaEquivocationOffence<Public>> for Pallet<T> {
    /// Report an `offence` and reward given `reporters`.
    fn report_offence(
      _: Vec<Public>,
      offence: GrandpaEquivocationOffence<Public>,
    ) -> Result<(), OffenceError> {
      // slash the offender
      let offender = offence.offender;
      Self::slash_validator(offender);

      // disable it
      Self::disable_validator(offender);

      Ok(())
    }

    fn is_known_offence(
      offenders: &[Public],
      _slot: &<GrandpaEquivocationOffence<Public> as Offence<Public>>::TimeSlot,
    ) -> bool {
      for offender in offenders {
        if Self::can_slash_validator(*offender) {
          return false;
        }
      }
      true
    }
  }

  impl<T: Config> FindAuthor<Public> for Pallet<T> {
    fn find_author<'a, I>(digests: I) -> Option<Public>
    where
      I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
      let i = Babe::<T>::find_author(digests)?;
      Some(Babe::<T>::authorities()[i as usize].0.clone().into())
    }
  }

  impl<T: Config> DisabledValidators for Pallet<T> {
    fn is_disabled(index: u32) -> bool {
      DisabledIndices::<T>::get(index).is_some()
    }
  }
}

pub use pallet::*;
