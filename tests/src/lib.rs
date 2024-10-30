use std::{
  time::{SystemTime, Duration},
  path::PathBuf,
  fs,
  process::Command,
};

use wikiblocks_client::{Wikiblocks, Transaction};

mod articles;

#[allow(dead_code)]
pub async fn publish_tx(wikiblocks: &Wikiblocks, tx: &Transaction) -> [u8; 32] {
  let mut latest = wikiblocks
    .block(wikiblocks.latest_finalized_block_hash().await.unwrap())
    .await
    .unwrap()
    .unwrap()
    .number();

  wikiblocks.publish(tx).await.unwrap();

  // Get the block it was included in
  // TODO: Add an RPC method for this/check the guarantee on the subscription
  let mut ticks = 0;
  loop {
    latest += 1;

    let block = {
      let mut block;
      while {
        block = wikiblocks.finalized_block_by_number(latest).await.unwrap();
        block.is_none()
      } {
        tokio::time::sleep(Duration::from_secs(1)).await;
        ticks += 1;

        if ticks > 60 {
          panic!("60 seconds without inclusion in a finalized block");
        }
      }
      block.unwrap()
    };

    for transaction in &block.transactions {
      if transaction == tx {
        return block.hash();
      }
    }
  }
}

pub fn docker_build() {
  let repo_path = std::path::PathBuf::from("/Users/akilbozbas/wikiblocks");
  let dockerfile_path =
    std::path::PathBuf::from("/Users/akilbozbas/wikiblocks/tests/docker/Dockerfile");

  // If this Docker image was created after this repo was last edited, return here
  // This should have better performance than Docker and allows running while offline
  if let Ok(res) = Command::new("docker")
    .arg("inspect")
    .arg("-f")
    .arg("{{ .Metadata.LastTagTime }}")
    .arg("wikiblocks-dev")
    .output()
  {
    let last_tag_time_buf = String::from_utf8(res.stdout).expect("docker had non-utf8 output");
    let last_tag_time = last_tag_time_buf.trim();
    if !last_tag_time.is_empty() {
      let created_time = SystemTime::from(
        chrono::DateTime::parse_and_remainder(last_tag_time, "%F %T.%f %z")
          .unwrap_or_else(|_| {
            panic!("docker formatted last tag time unexpectedly: {last_tag_time}")
          })
          .0,
      );

      // For all services, if the Dockerfile was edited after the image was built we should rebuild
      let mut last_modified =
        fs::metadata(&dockerfile_path).ok().and_then(|meta| meta.modified().ok());

      // Check any additionally specified paths
      let meta = |path: PathBuf| (path.clone(), fs::metadata(path));
      let mut metadatas = vec![meta(repo_path.join("src"))];

      while !metadatas.is_empty() {
        if let (path, Ok(metadata)) = metadatas.pop().unwrap() {
          if metadata.is_file() {
            if let Ok(modified) = metadata.modified() {
              if modified >
                last_modified
                  .expect("got when source was last modified yet not when the Dockerfile was")
              {
                last_modified = Some(modified);
              }
            }
          } else {
            // Recursively crawl since we care when the folder's contents were edited, not the
            // folder itself
            for entry in fs::read_dir(path.clone()).expect("couldn't read directory") {
              metadatas.push(meta(
                path.join(entry.expect("couldn't access item in directory").file_name()),
              ));
            }
          }
        }
      }

      if let Some(last_modified) = last_modified {
        if last_modified < created_time {
          println!("Node was built after the most recent source code edits, assuming built.");
          return;
        }
      }
    }
  }

  println!("Building ...");

  // Version which always prints
  if !Command::new("docker")
    .current_dir(&repo_path)
    .arg("build")
    .arg("-f")
    .arg(dockerfile_path)
    .arg(".")
    .arg("-t")
    .arg("wikiblocks-dev")
    .spawn()
    .unwrap()
    .wait()
    .unwrap()
    .success()
  {
    panic!("failed to build");
  }
  println!("Built!");
}

#[macro_export]
macro_rules! wikiblocks_test {
  ($($name: ident: $test: expr)*) => {
    $(
      #[tokio::test]
      async fn $name() {
        use std::collections::HashMap;
        use dockertest::{
          PullPolicy, StartPolicy, LogOptions, LogAction, LogPolicy, LogSource, Image,
          TestBodySpecification, DockerTest,
        };
        use wikiblocks_client::Wikiblocks;
        use $crate::docker_build;

        docker_build();

        let handle = concat!("wikiblocks_node-", stringify!($name));

        let composition = TestBodySpecification::with_image(
          Image::with_repository("wikiblocks-dev").pull_policy(PullPolicy::Never),
        )
        .replace_cmd(vec![
          "wikiblocks-node".to_string(),
          "--dev".to_string(),
          "--unsafe-rpc-external".to_string(),
          "--rpc-cors".to_string(),
          "all".to_string(),
        ])
        .replace_env(
          HashMap::from([
            ("RUST_LOG".to_string(), "runtime=debug".to_string()),
            ("KEY".to_string(), " ".to_string()),
          ])
        )
        .set_publish_all_ports(true)
        .set_handle(handle)
        .set_start_policy(StartPolicy::Strict)
        .set_log_options(Some(LogOptions {
          action: LogAction::Forward,
          policy: LogPolicy::Always,
          source: LogSource::Both,
        }));

        let mut test = DockerTest::new().with_network(dockertest::Network::Isolated);
        test.provide_container(composition);
        test.run_async(|ops| async move {
          // Sleep until the Substrate RPC starts
          let serai_rpc = ops.handle(handle).host_port(9944).unwrap();
          let serai_rpc = format!("http://{}:{}", serai_rpc.0, serai_rpc.1);
          // Bound execution to 60 seconds
          for _ in 0 .. 60 {
            tokio::time::sleep(core::time::Duration::from_secs(1)).await;
            let Ok(client) = Wikiblocks::new(serai_rpc.clone()).await else { continue };
            if client.latest_finalized_block_hash().await.is_err() {
              continue;
            }
            break;
          }
          #[allow(clippy::redundant_closure_call)]
          $test(Wikiblocks::new(serai_rpc).await.unwrap()).await;
        }).await;
      }
    )*
  }
}
