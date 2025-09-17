use std::{net::SocketAddr, sync::Arc};
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{StreamExt, SinkExt};
use tokio::{net::TcpListener, sync::broadcast};
use serde::{Serialize, Deserialize};

use crate::poh::Hash;
use crate::slot::Slot;

#[derive(Clone)]
pub struct NetState {
    pub node_id: String,
    pub tx: broadcast::Sender<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NetMsg {
    Hello { node_id: String },
    SlotBroadcast {prev_hash: Hash, slot: Slot },
    Ping,
}

impl NetMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize NetMsg")
    }
    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

pub fn router(state: NetState) -> Router {
    Router::new()
    .route("/ws", get(ws_handler))
    .with_state(std::sync::Arc::new(state))
}

async fn ws_handler(State(state): State<Arc<NetState>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| we_on_upgrade(state, socket))
}

async fn we_on_upgrade(state: Arc<NetState>, socket: WebSocket) {
    let hello = NetMsg::Hello { node_id: state.node_id.clone() }.to_json();

    let mut rx = state.tx.subscribe();

    let (mut ws_tx, mut ws_rx) = socket.split();

    let mut send_task = {
        let hello_clone = hello.clone();
        tokio::spawn(async move {
            let _ = ws_tx.send(WsMessage::Text(hello_clone)).await;
            loop {
                match rx.recv().await {
                    Ok(json) => { let _ = ws_tx.send(WsMessage::Text(json)).await; }
                    Err(broadcast::error::RecvError::Lagged(_)) => {/* 遅延時はスキップ */}
                    Err(_) => break,
                }
            }
        })
    };

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                WsMessage::Text(s) => {
                    if let Some(m) = NetMsg::from_json(&s) {
                        tracing::info!("received: {:?}", m);
                    }
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => { recv_task.abort(); }
        _ = (&mut recv_task) => { send_task.abort(); }
    }
}

pub async fn serve_ws(state: NetState, addr: SocketAddr) -> anyhow::Result<()> {
    let app = router(state);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WS server listening on{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn broadcast_slot(state: &NetState, prev_hash: Hash, slot: Slot) {
    let _ = state.tx.send(NetMsg::SlotBroadcast { prev_hash, slot }.to_json());
}