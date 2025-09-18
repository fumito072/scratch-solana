mod poh;
mod slot;
mod producer;
mod schedule;
mod driver;
mod net;

use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

use tokio::sync::mpsc;
use slot::{Slot, verify_slot};
use net::{NetMsg, NetState, serve_ws, broadcast_slot};

use poh::Hash;
use schedule::LeaderSchedule;
use driver::drive_round_robin;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (app_tx, mut app_rx) = mpsc::channel::<NetMsg>(1024);

    // 1) WSサーバを起動
    let (tx, _rx0) = tokio::sync::broadcast::channel::<String>(1024);
    let net_state = NetState { node_id: "B".into(), tx, app_tx };
    let ws_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    tokio::spawn({
        let st = net_state.clone();
        async move {
            let _ = serve_ws(st, ws_addr).await;
        }
    });

    let mut tip: Hash = [0u8; 32];
    let mut adopted: Vec<Slot> = Vec::new();

    let schedule = LeaderSchedule::new(vec!["A".into(), "B".into(), "C".into()]).unwrap();
    let my_id = "B";

    let mut s = 0u64;
    let app_tx_for_producer = net_state.app_tx.clone();
    tokio::spawn({
        let net_state = net_state.clone();
        let schedule = schedule.clone();
        async move {
            let mut prev = tip; // HashはCopy
            while s < 6 {
                let chain = drive_round_robin(&schedule, my_id, prev, s, 1);
                let made = chain[0].clone();

                broadcast_slot(&net_state, prev, made.clone());

                let _ = app_tx_for_producer
                    .send(NetMsg::SlotBroadcast { prev_hash: prev, slot: made.clone() })
                    .await;

                prev = made.last_hash;
                s += 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    });

    while let Some(msg) = app_rx.recv().await {
        match msg {
            NetMsg::SlotBroadcast { prev_hash, slot } => {
                if prev_hash == tip && verify_slot(&tip, &slot) {
                    tracing::info!("adopted slot={} entries={}", slot.slot, slot.entries.len());
                    tip = slot.last_hash;
                    adopted.push(slot);
                } else {
                    tracing::warn!("ignored slot={} (prev mismatch or verify failed)", slot.slot);
                }
            }
            NetMsg::Hello { node_id } => {
                tracing::info!("peer hello: {}", node_id);
            }
            NetMsg::Ping => {}
        }
    }

    Ok(())
}
