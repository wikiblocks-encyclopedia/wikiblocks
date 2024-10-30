#![cfg_attr(not(feature = "std"), no_std)]

use wikiblocks_primitives::SubstrateAmount;

pub trait CallToFee<T: frame_system::Config> {
  fn call_to_fee(call: &T::RuntimeCall) -> SubstrateAmount;
}

// TODO: Investigate why Substrate generates this
#[allow(unreachable_patterns, clippy::cast_possible_truncation)]
#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use sp_std::vec::Vec;
  use sp_core::sr25519::Public;
  use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf},
    transaction_validity::{TransactionValidityError, InvalidTransaction},
  };

  use frame_system::pallet_prelude::*;
  use frame_support::pallet_prelude::*;

  use pallet_transaction_payment::{Config as TpConfig, OnChargeTransaction};

  pub use coins_primitives as primitives;
  use primitives::*;

  #[pallet::config]
  pub trait Config: frame_system::Config<AccountId = Public> {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type CallToFee: CallToFee<Self>;
  }

  #[pallet::genesis_config]
  #[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
  pub struct GenesisConfig<T: Config> {
    pub accounts: Vec<(T::AccountId, SubstrateAmount)>,
    pub _ignore: PhantomData<T>,
  }

  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      GenesisConfig { accounts: Default::default(), _ignore: Default::default() }
    }
  }

  #[pallet::error]
  pub enum Error<T> {
    AmountOverflowed,
    NotEnoughCoins,
    MintNotAllowed,
  }

  #[pallet::event]
  #[pallet::generate_deposit(fn deposit_event)]
  pub enum Event<T: Config> {
    Mint { to: Public, amount: SubstrateAmount },
    Burn { from: Public, amount: SubstrateAmount },
    Transfer { from: Public, to: Public, amount: SubstrateAmount },
  }

  #[pallet::pallet]
  pub struct Pallet<T>(_);

  /// The amount of coins each account has.
  // Identity is used as the second key's hasher due to it being a non-manipulatable fixed-space
  // ID.
  #[pallet::storage]
  #[pallet::getter(fn balances)]
  pub type Balances<T: Config> =
    StorageMap<_, Blake2_128Concat, Public, SubstrateAmount, OptionQuery>;

  /// The total supply of each coin.
  // We use Identity type here again due to reasons stated in the Balances Storage.
  #[pallet::storage]
  #[pallet::getter(fn supply)]
  pub type Supply<T: Config> = StorageValue<_, SubstrateAmount, ValueQuery>;

  #[pallet::genesis_build]
  impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
    fn build(&self) {
      Supply::<T>::set(0);
      // initialize the genesis accounts
      for (account, balance) in &self.accounts {
        Pallet::<T>::mint(*account, *balance).unwrap();
      }
    }
  }

  impl<T: Config> Pallet<T> {
    fn decrease_balance_internal(from: Public, amount: SubstrateAmount) -> Result<(), Error<T>> {
      // sub amount from account
      let new_amount = Self::balances(from)
        .ok_or(Error::<T>::NotEnoughCoins)?
        .checked_sub(amount)
        .ok_or(Error::<T>::NotEnoughCoins)?;

      // save
      if new_amount == 0 {
        Balances::<T>::remove(from);
      } else {
        Balances::<T>::set(from, Some(new_amount));
      }
      Ok(())
    }

    fn increase_balance_internal(to: Public, amount: SubstrateAmount) -> Result<(), Error<T>> {
      // add amount to account
      let new_amount =
        Self::balances(to).unwrap_or(0).checked_add(amount).ok_or(Error::<T>::AmountOverflowed)?;

      // save
      Balances::<T>::set(to, Some(new_amount));
      Ok(())
    }

    /// Mint `balance` to the given account.
    ///
    /// Errors if any amount overflows.
    pub fn mint(to: Public, amount: SubstrateAmount) -> Result<(), Error<T>> {
      // update the balance
      Self::increase_balance_internal(to, amount)?;

      // update the supply
      let new_supply = Self::supply().checked_add(amount).ok_or(Error::<T>::AmountOverflowed)?;
      Supply::<T>::set(new_supply);

      Self::deposit_event(Event::Mint { to, amount });
      Ok(())
    }

    /// Burn `balance` from the specified account.
    fn burn_internal(from: Public, amount: SubstrateAmount) -> Result<(), Error<T>> {
      // don't waste time if amount == 0
      if amount == 0 {
        return Ok(());
      }

      // update the balance
      Self::decrease_balance_internal(from, amount)?;

      // update the supply
      let new_supply = Self::supply().checked_sub(amount).unwrap();
      Supply::<T>::set(new_supply);

      Ok(())
    }

    /// Transfer `balance` from `from` to `to`.
    pub fn transfer_internal(
      from: Public,
      to: Public,
      amount: SubstrateAmount,
    ) -> Result<(), Error<T>> {
      // update balances of accounts
      Self::decrease_balance_internal(from, amount)?;
      Self::increase_balance_internal(to, amount)?;
      Self::deposit_event(Event::Transfer { from, to, amount });
      Ok(())
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn transfer(origin: OriginFor<T>, to: Public, amount: SubstrateAmount) -> DispatchResult {
      let from = ensure_signed(origin)?;
      Self::transfer_internal(from, to, amount)?;
      Ok(())
    }

    /// Burn `balance` from the caller.
    #[pallet::call_index(1)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn burn(origin: OriginFor<T>, amount: SubstrateAmount) -> DispatchResult {
      let from = ensure_signed(origin)?;
      Self::burn_internal(from, amount)?;
      Self::deposit_event(Event::Burn { from, amount });
      Ok(())
    }
  }

  impl<T: Config> OnChargeTransaction<T> for Pallet<T>
  where
    T: TpConfig,
  {
    type Balance = SubstrateAmount;
    type LiquidityInfo = Option<SubstrateAmount>;

    fn withdraw_fee(
      who: &Public,
      call: &T::RuntimeCall,
      _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
      fee: Self::Balance,
      tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
      if fee == 0 {
        return Ok(None);
      }

      // check we have the right amount of fee for the call
      let amount = T::CallToFee::call_to_fee(call);
      if tip < amount {
        Err(InvalidTransaction::Payment)?;
      }

      match Self::transfer_internal(*who, FEE_ACCOUNT.into(), fee) {
        Err(_) => Err(InvalidTransaction::Payment)?,
        Ok(()) => Ok(Some(fee)),
      }
    }

    fn correct_and_deposit_fee(
      who: &Public,
      _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
      _post_info: &PostDispatchInfoOf<T::RuntimeCall>,
      corrected_fee: Self::Balance,
      _tip: Self::Balance,
      already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
      if let Some(paid) = already_withdrawn {
        let refund_amount = paid.saturating_sub(corrected_fee);
        Self::transfer_internal(FEE_ACCOUNT.into(), *who, refund_amount)
          .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
      }
      Ok(())
    }
  }
}

pub use pallet::*;
