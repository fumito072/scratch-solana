mod poh;
mod slot;
mod producer;
mod schedule;
mod driver;
mod net;

use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

use poh::Hash;
use schedule::LeaderSchedule;
use driver::drive_round_robin;
use net::{NetState, serve_ws, broadcast_slot};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // 1) WSサーバを起動
    let (tx, _rx0) = tokio::sync::broadcast::channel::<String>(1024);
    let net_state = NetState { node_id: "B".into(), tx };
    let ws_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let ws_state_clone = net_state.clone();
    tokio::spawn(async move {
        let _ = serve_ws(ws_state_clone, ws_addr).await;
    });

    // 2) これまでのドライバでSlotを生成（デモ：生成したら毎回broadcast）
    let zero: Hash = [0u8; 32];
    let schedule = LeaderSchedule::new(vec!["A".into(), "B".into(), "C".into()]).unwrap();
    let my_id = "B";

    // デモ：1スロットずつ前進して配信
    let mut prev = zero;
    let mut s = 0u64;
    while s < 6 {
        let chain = drive_round_robin(&schedule, my_id, prev, s, 1); // 1スロットだけ作る
        let made = &chain[0];
        // 配信（ここでは自ノード生成分も他ノード模擬分も配る）
        broadcast_slot(&net_state, prev, made.clone());
        prev = made.last_hash;
        s += 1;
        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
