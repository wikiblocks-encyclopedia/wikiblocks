#![cfg_attr(not(feature = "std"), no_std)]

#[frame_support::pallet]
pub mod pallet {
  use frame_system::pallet_prelude::*;
  use frame_support::pallet_prelude::*;

  use sp_core::sr25519::Public;

  use wikiblocks_primitives::{ArticleVersion, Body, OpCode, Script, Title};

  #[pallet::config]
  pub trait Config: frame_system::Config<AccountId = Public> {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
  }

  #[pallet::error]
  pub enum Error<T> {
    InvalidScript,
    TitleAlreadyExist,
    InvalidTitle,
    StorageFull,
  }

  #[pallet::event]
  pub enum Event<T: Config> {}

  #[pallet::pallet]
  pub struct Pallet<T>(PhantomData<T>);

  #[pallet::storage]
  #[pallet::getter(fn titles)]
  pub type Titles<T: Config> = StorageValue<_, BoundedVec<Title, ConstU32<1_000_000>>, ValueQuery>;

  /// Stores the last article version. If this returns Let's say `ArticleVersion(5)` that means
  /// there are versions 0, 1, 2, 3, 4, 5 for the article.
  #[pallet::storage]
  #[pallet::getter(fn versions)]
  pub type Versions<T: Config> =
    StorageMap<_, Blake2_128Concat, Title, ArticleVersion, OptionQuery>;

  #[pallet::storage]
  #[pallet::getter(fn articles)]
  pub type Articles<T: Config> =
    StorageMap<_, Blake2_128Concat, (Title, ArticleVersion), Script, OptionQuery>;

  #[pallet::storage]
  #[pallet::getter(fn authors)]
  pub type Authors<T: Config> =
    StorageMap<_, Blake2_128Concat, (Title, ArticleVersion), Public, OptionQuery>;

  impl<T: Config> Pallet<T> {
    // TODO: this can be optimized to O(1) using maps
    fn title_exist(title: &Title) -> bool {
      Self::titles().iter().find(|&t| t == title).is_some()
    }

    fn validate_add_article_script(script: &Script) -> Result<(Title, Body), Error<T>> {
      // first Opcode should be the Title
      let mut iter = script.data().iter();
      let opcode = iter.next().ok_or(Error::<T>::InvalidScript)?;
      let OpCode::Title(title) = opcode else {
        return Err(Error::<T>::InvalidScript);
      };

      // second Opcode should add a body
      let opcode = iter.next().ok_or(Error::<T>::InvalidScript)?;
      let OpCode::Add(body) = opcode else {
        return Err(Error::<T>::InvalidScript);
      };

      // add article Script should have a Title and Body Opcodes only
      if iter.next().is_some() {
        Err(Error::<T>::InvalidScript)?;
      }

      if title.data().is_empty() {
        Err(Error::<T>::InvalidScript)?;
      }

      if body.data().is_empty() {
        Err(Error::<T>::InvalidScript)?;
      }

      // check title doesn't already exist
      if Self::title_exist(title) {
        Err(Error::<T>::TitleAlreadyExist)?;
      }

      Ok((title.clone(), body.clone()))
    }

    fn validate_add_version_script(title: &Title, script: &Script) -> Result<(), Error<T>> {
      // script can't be empty
      if script.data().is_empty() {
        Err(Error::<T>::InvalidScript)?;
      }

      // make sure the title exist
      if !Self::title_exist(title) {
        Err(Error::<T>::InvalidTitle)?;
      }

      // add version script can't have Title opcode
      if script.data().iter().find(|&code| matches!(code, OpCode::Title(..))).is_some() {
        Err(Error::<T>::InvalidScript)?;
      }

      Ok(())
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn add_article(origin: OriginFor<T>, script: Script) -> DispatchResult {
      let from = ensure_signed(origin)?;

      // TODO: check the total "data" within the script. It should be as big as tx size
      // at the moment.

      // TODO: Did the fee for this article been collected? Check that fee amount is right.

      // TODO: we should prevent the cloning here
      let (title, body) = Self::validate_add_article_script(&script)?;

      // insert the title
      if Titles::<T>::try_append(&title).is_err() {
        Err(Error::<T>::StorageFull)?;
      }

      // insert a version
      Versions::<T>::set(&title, Some(0));

      // insert the body
      Articles::<T>::set((&title, 0), Some(Script::new(vec![OpCode::Add(body)]).unwrap()));

      // insert the author
      Authors::<T>::set((title, 0), Some(from));
      Ok(())
    }

    #[pallet::call_index(1)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn add_version(origin: OriginFor<T>, title: Title, script: Script) -> DispatchResult {
      let from = ensure_signed(origin)?;

      // TODO: check the total "data" within the script. It should be as big as tx size
      // at the moment.

      // TODO: Did the fee for this article been collected? Check that fee amount is right.
      Self::validate_add_version_script(&title, &script)?;

      // update the versions
      // we can unwrap here since we pass the validation, so we have the title hence a version
      let version = Self::versions(&title).unwrap() + 1;
      Versions::<T>::set(&title, Some(version));

      // insert the body for the version
      Articles::<T>::set((&title, version), Some(script));

      // insert the author
      Authors::<T>::set((title, version), Some(from));
      Ok(())
    }
  }
}

pub use pallet::*;
