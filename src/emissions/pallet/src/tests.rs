use crate::mock::*;

use frame_support::traits::{Hooks, Get};

use validator_sets_pallet::{Pallet as ValidatorSets, primitives::Session};

use wikiblocks_primitives::*;

#[test]
fn check_post_ec_security_emissions() {
  new_test_ext().execute_with(|| {
    let mut block_number = System::block_number();

    // update TAS
    ValidatorSets::<Test>::new_session();
    ValidatorSets::<Test>::retire_set(Session(0));

    // move the block number for the next session
    block_number += <<Test as pallet_babe::Config>::EpochDuration as Get<u64>>::get();
    System::set_block_number(block_number);

    for _ in 0 .. 5 {
      // get current stakes & each pool SRI amounts
      let current_stake = ValidatorSets::<Test>::total_allocated_stake().unwrap();

      // trigger rewards distribution for the past session
      ValidatorSets::<Test>::new_session();
      <Emissions as Hooks<BlockNumber>>::on_initialize(block_number + 1);

      // calculate the total reward for this epoch
      let session = ValidatorSets::<Test>::session().unwrap_or(Session(0));
      let block_count = ValidatorSets::<Test>::session_begin_block(session) -
        ValidatorSets::<Test>::session_begin_block(Session(session.0 - 1));
      let reward_this_epoch = block_count * REWARD_PER_BLOCK;

      ValidatorSets::<Test>::retire_set(Session(session.0 - 1));

      // all validator rewards should automatically be staked
      assert_eq!(
        ValidatorSets::<Test>::total_allocated_stake().unwrap(),
        current_stake + reward_this_epoch
      );

      block_number += <<Test as pallet_babe::Config>::EpochDuration as Get<u64>>::get();
      System::set_block_number(block_number);
    }
  });
}
