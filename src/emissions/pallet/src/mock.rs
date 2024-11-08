//! Test environment for Emissions pallet.

use super::*;

use frame_support::{
  construct_runtime,
  traits::{ConstU32, ConstU64},
};

use sp_core::{H256, Pair, sr25519::Public};
use sp_runtime::{
  traits::{BlakeTwo256, IdentityLookup},
  BuildStorage,
};

use wikiblocks_primitives::*;
use validator_sets::{primitives::MAX_KEY_SHARES_PER_SET, MembershipProof};

use crate as emissions;
pub use coins_pallet as coins;
pub use validator_sets_pallet as validator_sets;
pub use pallet_babe as babe;
pub use pallet_grandpa as grandpa;
pub use pallet_timestamp as timestamp;

type Block = frame_system::mocking::MockBlock<Test>;
// Maximum number of authorities per session.
pub type MaxAuthorities = ConstU32<{ MAX_KEY_SHARES_PER_SET }>;

construct_runtime!(
  pub enum Test
  {
    System: frame_system,
    Timestamp: timestamp,
    Coins: coins,
    Emissions: emissions,
    ValidatorSets: validator_sets,
    Babe: babe,
    Grandpa: grandpa,
  }
);

impl frame_system::Config for Test {
  type BaseCallFilter = frame_support::traits::Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type RuntimeOrigin = RuntimeOrigin;
  type RuntimeCall = RuntimeCall;
  type Nonce = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = Public;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Block = Block;
  type RuntimeEvent = RuntimeEvent;
  type BlockHashCount = ConstU64<250>;
  type DbWeight = ();
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = ();
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = ();
  type OnSetCode = ();
  type MaxConsumers = ConstU32<16>;
}

impl timestamp::Config for Test {
  type Moment = u64;
  type OnTimestampSet = Babe;
  type MinimumPeriod = ConstU64<{ (TARGET_BLOCK_TIME * 1000) / 2 }>;
  type WeightInfo = ();
}

impl babe::Config for Test {
  type EpochDuration = ConstU64<{ FAST_EPOCH_DURATION }>;

  type ExpectedBlockTime = ConstU64<{ TARGET_BLOCK_TIME * 1000 }>;
  type EpochChangeTrigger = babe::ExternalTrigger;
  type DisabledValidators = ValidatorSets;

  type WeightInfo = ();
  type MaxAuthorities = MaxAuthorities;

  type KeyOwnerProof = MembershipProof<Self>;
  type EquivocationReportSystem = ();
}

impl grandpa::Config for Test {
  type RuntimeEvent = RuntimeEvent;

  type WeightInfo = ();
  type MaxAuthorities = MaxAuthorities;

  type MaxSetIdSessionEntries = ConstU64<0>;
  type KeyOwnerProof = MembershipProof<Self>;
  type EquivocationReportSystem = ();
}

pub struct FeeCollector;
impl coins::CallToFee<Test> for FeeCollector {
  fn call_to_fee(_: &RuntimeCall) -> SubstrateAmount {
    0
  }
}

impl coins::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type CallToFee = FeeCollector;
}

impl validator_sets::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type ShouldEndSession = Babe;
}

impl Config for Test {
  type RuntimeEvent = RuntimeEvent;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
  let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

  let accounts: Vec<Public> = vec![
    insecure_pair_from_name("Alice").public(),
    insecure_pair_from_name("Bob").public(),
    insecure_pair_from_name("Charlie").public(),
    insecure_pair_from_name("Dave").public(),
    insecure_pair_from_name("Eve").public(),
    insecure_pair_from_name("Ferdie").public(),
  ];
  let key_share_amount = 50_000 * 10_u64.pow(8);
  let validators = accounts.clone().into_iter().map(|a| (a, key_share_amount)).collect::<Vec<_>>();

  coins::GenesisConfig::<Test> {
    accounts: accounts.into_iter().map(|a| (a, 1 << 60)).collect(),
    _ignore: Default::default(),
  }
  .assimilate_storage(&mut t)
  .unwrap();

  validator_sets::GenesisConfig::<Test> { participants: validators.clone(), key_share_amount }
    .assimilate_storage(&mut t)
    .unwrap();

  crate::GenesisConfig::<Test> { participants: validators.clone() }
    .assimilate_storage(&mut t)
    .unwrap();

  let mut ext = sp_io::TestExternalities::new(t);
  ext.execute_with(|| System::set_block_number(0));
  ext
}
