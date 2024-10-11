#[cfg(feature = "serai")]
mod serai;
#[cfg(feature = "serai")]
pub use serai::*;

#[cfg(not(feature = "serai"))]
pub use serai_abi::primitives;
#[cfg(not(feature = "serai"))]
mod other_primitives {
  pub mod coins {
    pub use serai_abi::coins::primitives;
  }
  pub mod validator_sets {
    pub use serai_abi::validator_sets::primitives;
  }
}
#[cfg(not(feature = "serai"))]
pub use other_primitives::*;
