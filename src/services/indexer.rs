use crate::state::SharedState;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run_indexer(state: SharedState) {
    println!("Solana chain indexer worker active...");
    loop {
        // Poll Solana RPC or Yellowstone gRPC here and update Postgres rounds/deployments
        let _ = &state.solana_rpc;
        sleep(Duration::from_secs(5)).await;
    }
}
