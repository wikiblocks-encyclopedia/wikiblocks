#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[allow(clippy::cast_possible_truncation)]
#[frame_support::pallet]
pub mod pallet {
  use frame_system::pallet_prelude::*;
  use frame_support::pallet_prelude::*;

  use sp_core::sr25519::Public;
  use sp_std::vec;

  use wikiblocks_primitives::{ArticleVersion, Body, OpCode, Script, Title, Article, MAX_DATA_LEN};

  #[pallet::config]
  pub trait Config: frame_system::Config<AccountId = Public> {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
  }

  #[pallet::error]
  pub enum Error<T> {
    InvalidScript,
    TitleAlreadyExist,
    InvalidTitle,
    InvalidReference,
    StorageFull,
    TooManyVersions,
    ArticleTooBig,
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
  #[pallet::getter(fn last_version)]
  pub type LastVersion<T: Config> =
    StorageMap<_, Blake2_128Concat, Title, ArticleVersion, OptionQuery>;

  #[pallet::storage]
  #[pallet::getter(fn articles)]
  pub type Articles<T: Config> = StorageMap<_, Blake2_128Concat, Article, Script, OptionQuery>;

  #[pallet::storage]
  #[pallet::getter(fn authors)]
  pub type Authors<T: Config> = StorageMap<_, Blake2_128Concat, Article, Public, OptionQuery>;

  impl<T: Config> Pallet<T> {
    // TODO: this can be optimized to O(1) using maps
    pub fn title_exist(title: &Title) -> bool {
      Self::titles().iter().any(|t| t == title)
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
      // check the total "data" within the script.
      if script.encode().len() > usize::try_from(MAX_DATA_LEN).unwrap() {
        Err(Error::<T>::ArticleTooBig)?;
      }

      // script can't be empty
      if script.data().is_empty() {
        Err(Error::<T>::InvalidScript)?;
      }

      // make sure the title exist
      if !Self::title_exist(title) {
        Err(Error::<T>::InvalidTitle)?;
      }

      // verify the opcodes
      let last_version = Self::last_version(title).ok_or(Error::<T>::InvalidTitle)?;
      let mut reference_in_hand = false;
      for opcode in script.data() {
        // add version script can't have a Title opcode
        if matches!(opcode, OpCode::Title(..)) {
          Err(Error::<T>::InvalidScript)?;
        }

        // make sure all reference versions exist
        if let OpCode::Reference(v) = opcode {
          if v.0 > last_version.0 {
            Err(Error::<T>::InvalidReference)?;
          }
          reference_in_hand = true;
        }

        // if we have an opcode that requires reference to work on, we must have a reference
        if opcode.requires_reference() && !reference_in_hand {
          Err(Error::<T>::InvalidScript)?;
        }
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

      // TODO: we should prevent the cloning here
      let (title, body) = Self::validate_add_article_script(&script)?;

      // try inserting the title
      if Titles::<T>::try_append(&title).is_err() {
        Err(Error::<T>::StorageFull)?;
      }

      // make the article
      let article = Article::new(title, ArticleVersion(0));

      // update last version
      LastVersion::<T>::set(article.title(), Some(article.version()));

      // insert the body
      Articles::<T>::set(&article, Some(Script::new(vec![OpCode::Add(body)]).unwrap()));

      // insert the author
      Authors::<T>::set(article, Some(from));
      Ok(())
    }

    #[pallet::call_index(1)]
    #[pallet::weight((0, DispatchClass::Normal))] // TODO
    pub fn add_version(origin: OriginFor<T>, title: Title, script: Script) -> DispatchResult {
      let from = ensure_signed(origin)?;

      // validate the script
      Self::validate_add_version_script(&title, &script)?;

      // update the versions
      // we can unwrap here since we pass the validation, so we have the title hence a version
      let version = ArticleVersion(
        Self::last_version(&title).unwrap().0.checked_add(1).ok_or(Error::<T>::TooManyVersions)?,
      );

      // construct the article
      let article = Article::new(title, version);

      // update last version
      LastVersion::<T>::set(article.title(), Some(article.version()));

      // insert the body for the version
      Articles::<T>::set(&article, Some(script));

      // insert the author
      Authors::<T>::set(article, Some(from));
      Ok(())
    }
  }
}

pub use pallet::*;
