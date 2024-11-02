#[allow(unused_imports)]
use crate::{docker_build, allocate_stake, deallocate_stake};

#[allow(unused_imports)]
use sp_core::{
  sr25519::{Public, Pair},
  Pair as PairTrait,
};

#[allow(unused_imports)]
use wikiblocks_client::{
  primitives::{insecure_pair_from_name, TARGET_BLOCK_TIME, FAST_EPOCH_DURATION},
  validator_sets::{ValidatorSetsEvent, primitives::Session},
  Wikiblocks,
};

#[tokio::test]
async fn validator_set_rotation() {
  use dockertest::{
    PullPolicy, StartPolicy, LogOptions, LogAction, LogPolicy, LogSource, Image,
    TestBodySpecification, DockerTest,
  };
  use std::collections::HashMap;

  docker_build();

  let handle = |name| format!("wikiblocks_node-{name}");
  let composition = |name| {
    TestBodySpecification::with_image(
      Image::with_repository("wikiblocks-dev").pull_policy(PullPolicy::Never),
    )
    .replace_cmd(vec![
      "wikiblocks-node".to_string(),
      "--unsafe-rpc-external".to_string(),
      "--rpc-cors".to_string(),
      "all".to_string(),
      "--chain".to_string(),
      "local".to_string(),
      format!("--{name}"),
    ])
    .replace_env(HashMap::from([
      ("RUST_LOG".to_string(), "runtime=debug".to_string()),
      ("KEY".to_string(), " ".to_string()),
    ]))
    .set_publish_all_ports(true)
    .set_handle(handle(name))
    .set_start_policy(StartPolicy::Strict)
    .set_log_options(Some(LogOptions {
      action: LogAction::Forward,
      policy: LogPolicy::Always,
      source: LogSource::Both,
    }))
  };

  let mut test = DockerTest::new().with_network(dockertest::Network::Isolated);
  test.provide_container(composition("alice"));
  test.provide_container(composition("bob"));
  test.provide_container(composition("charlie"));
  test.provide_container(composition("dave"));
  test.provide_container(composition("eve"));
  test
    .run_async(|ops| async move {
      // Sleep until the Substrate RPC starts
      let alice = handle("alice");
      let alice_rpc = ops.handle(&alice).host_port(9944).unwrap();
      let alice_rpc = format!("http://{}:{}", alice_rpc.0, alice_rpc.1);

      // Sleep for some time
      tokio::time::sleep(core::time::Duration::from_secs(20)).await;
      let wikiblocks = Wikiblocks::new(alice_rpc.clone()).await.unwrap();

      // Make sure the genesis is as expected
      assert_eq!(
        wikiblocks
          .as_of(wikiblocks.finalized_block_by_number(0).await.unwrap().unwrap().hash())
          .validator_sets()
          .new_session_events()
          .await
          .unwrap(),
        vec![ValidatorSetsEvent::NewSession { session: Session(0) }]
      );

      // genesis accounts
      let accounts = vec![
        insecure_pair_from_name("Alice"),
        insecure_pair_from_name("Bob"),
        insecure_pair_from_name("Charlie"),
        insecure_pair_from_name("Dave"),
        insecure_pair_from_name("Eve"),
      ];

      // amounts for single key share per network
      let key_share = 50_000 * 10_u64.pow(8);

      // genesis participants per network
      #[allow(clippy::redundant_closure_for_method_calls)]
      let default_participants =
        accounts[.. 4].to_vec().iter().map(|pair| pair.public()).collect::<Vec<_>>();
      let mut participants = default_participants.clone();

      // test the set rotation
      // we start the chain with 4 default participants that has a single key share each
      participants.sort();
      verify_session_and_active_validators(&wikiblocks, 0, &participants).await;

      // add 1 participant
      let last_participant = accounts[4].clone();
      let hash = allocate_stake(&wikiblocks, key_share, &last_participant, 0).await;
      participants.push(last_participant.public());
      // the session at which set changes becomes active
      let activation_session = get_session_at_which_changes_activate(&wikiblocks, hash).await;

      // verify
      participants.sort();
      verify_session_and_active_validators(&wikiblocks, activation_session, &participants).await;

      // remove 1 participant
      let participant_to_remove = accounts[1].clone();
      let hash = deallocate_stake(&wikiblocks, key_share, &participant_to_remove, 0).await;
      participants.swap_remove(
        participants.iter().position(|k| *k == participant_to_remove.public()).unwrap(),
      );
      let activation_session = get_session_at_which_changes_activate(&wikiblocks, hash).await;

      // verify
      participants.sort();
      verify_session_and_active_validators(&wikiblocks, activation_session, &participants).await;

      // check pending deallocations
      let pending = wikiblocks
        .as_of_latest_finalized_block()
        .await
        .unwrap()
        .validator_sets()
        .pending_deallocations(participant_to_remove.public(), Session(activation_session + 1))
        .await
        .unwrap();
      assert_eq!(pending, Some(key_share));
    })
    .await;
}

#[allow(dead_code)]
async fn session_for_block(wikiblocks: &Wikiblocks, block: [u8; 32]) -> u32 {
  wikiblocks.as_of(block).validator_sets().session().await.unwrap().unwrap().0
}

#[allow(dead_code)]
async fn verify_session_and_active_validators(
  serai: &Wikiblocks,
  session: u32,
  participants: &[Public],
) {
  // wait until the active session.
  let block = tokio::time::timeout(
    core::time::Duration::from_secs(FAST_EPOCH_DURATION * TARGET_BLOCK_TIME * 2),
    async move {
      loop {
        let mut block = serai.latest_finalized_block_hash().await.unwrap();
        if session_for_block(serai, block).await < session {
          // Sleep a block
          tokio::time::sleep(core::time::Duration::from_secs(TARGET_BLOCK_TIME)).await;
          continue;
        }
        while session_for_block(serai, block).await > session {
          block = serai.block(block).await.unwrap().unwrap().header.parent_hash.0;
        }
        assert_eq!(session_for_block(serai, block).await, session);
        break block;
      }
    },
  )
  .await
  .unwrap();
  let serai_for_block = serai.as_of(block);

  // verify session
  let s = serai_for_block.validator_sets().session().await.unwrap().unwrap();
  assert_eq!(s.0, session);

  // verify participants
  let mut validators = serai_for_block.validator_sets().active_network_validators().await.unwrap();
  validators.sort();
  assert_eq!(validators, participants);

  // make sure finalization continues as usual after the changes
  let current_finalized_block = serai.latest_finalized_block().await.unwrap().header.number;
  tokio::time::timeout(core::time::Duration::from_secs(TARGET_BLOCK_TIME * 10), async move {
    let mut finalized_block = serai.latest_finalized_block().await.unwrap().header.number;
    while finalized_block <= current_finalized_block + 2 {
      tokio::time::sleep(core::time::Duration::from_secs(TARGET_BLOCK_TIME)).await;
      finalized_block = serai.latest_finalized_block().await.unwrap().header.number;
    }
  })
  .await
  .unwrap();

  // TODO: verify key shares as well?
}

// changes should be active in the next session
// it takes 1 extra session for serai net to make the changes active.
#[allow(dead_code)]
async fn get_session_at_which_changes_activate(wikiblocks: &Wikiblocks, hash: [u8; 32]) -> u32 {
  session_for_block(wikiblocks, hash).await + 2
}
