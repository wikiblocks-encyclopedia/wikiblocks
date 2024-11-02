#![cfg_attr(not(feature = "std"), no_std)]

#[allow(clippy::cast_possible_truncation)]
#[frame_support::pallet]
pub mod pallet {
  use frame_system::pallet_prelude::*;
  use frame_support::pallet_prelude::*;

  use sp_core::sr25519::Public;
  use sp_std::vec;

  use articles_pallet::{Config as ArticlesConfig, Pallet as Articles};
  use wikiblocks_primitives::{ArticleVersion, Title};

  #[pallet::config]
  pub trait Config: frame_system::Config<AccountId = Public> + ArticlesConfig {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
  }

  #[pallet::error]
  pub enum Error<T> {
    InvalidTitle,
    InvalidVersion,
    TooManyUpvotes,
  }

  #[pallet::event]
  pub enum Event<T: Config> {}

  #[pallet::pallet]
  pub struct Pallet<T>(_);

  // TODO: have a new type for (Title, ArticleVersion).
  #[pallet::storage]
  #[pallet::getter(fn upvotes)]
  pub type Upvotes<T: Config> =
    StorageMap<_, Blake2_128Concat, (Title, ArticleVersion), u64, ValueQuery>;

  impl<T: Config> Pallet<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn upvote(origin: OriginFor<T>, title: Title, version: ArticleVersion) -> DispatchResult {
      let _ = ensure_signed(origin)?;

      // make sure title exist
      if !Articles::<T>::title_exist(&title) {
        Err(Error::<T>::InvalidTitle)?;
      }

      // make sure version exist
      let Some(last_version) = Articles::<T>::last_version(&title) else {
        return Err(Error::<T>::InvalidVersion)?;
      };
      if version.0 > last_version.0 {
        Err(Error::<T>::InvalidVersion)?;
      }

      // update the upvotes
      let current = Upvotes::<T>::get((&title, version));
      Upvotes::<T>::set(
        (title, version),
        current.checked_add(1).ok_or(Error::<T>::TooManyUpvotes)?,
      );

      Ok(())
    }
  }
}

pub use pallet::*;
