use std::{net::SocketAddr, sync::{Mutex, atomic::{AtomicU64, AtomicUsize}}};

use axum::{
    body::Bytes, extract::{ConnectInfo, State, ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade}}, response::IntoResponse
};
use axum_extra::{TypedHeader, headers};
use kameo::actor::ActorRef;

use crate::{actors::{Hub, WebClient}, protocols::PodId};

#[derive(Clone)]
pub struct AppState {
    pub actor_ref: ActorRef<Hub>,
    // pub next_id: std::sync::Arc<AtomicU64>,
    pub incrementor: std::sync::Arc<Mutex<Incrementor>>,
}

pub struct Incrementor {
    i: PodId,
}
impl Incrementor {
    pub fn new() -> Self{
        Incrementor{
            i:0,
        }
    }
    pub fn increment(&mut self) -> PodId {
        let id = self.i;
        self.i = self.i.checked_add(1).expect("we ran out of ids");
        id
    }
}

pub async fn websocket_handler(State(state): State<AppState>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    //let id = state.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let id = state.incrementor.lock().unwrap().increment();
    let web_actor = WebClient{
        id,
        hub: state.actor_ref.clone(),
        is_pod: false,
    };

    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    ws.on_failed_upgrade(|error| println!("Error upgrading websocket: {}", error))
        .on_upgrade(move |socket| handle_web_socket(socket, addr, web_actor))
}
async fn send_close_message(mut socket: WebSocket, code: u16, reason: &str) {
    _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await;
}
async fn handle_web_socket(mut socket: WebSocket, who: SocketAddr, web_actor: WebClient) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }
    // Returns `None` if the stream has closed.
    // debugging websocket connection
    while let Some(msg) = socket.recv().await {
        // TODO: process messages in extra function
        if let Ok(msg) = msg {
            match msg {
                Message::Text(utf8_bytes) => {
                    println!("Text received: {}", utf8_bytes);
                    let result = socket
                        .send(Message::Text(
                            format!("Echo back text: {}", utf8_bytes).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                Message::Binary(bytes) => {
                    println!("Received bytes of length: {}", bytes.len());
                    let result = socket
                        .send(Message::Text(
                            format!("Received bytes of length: {}", bytes.len()).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        println!(
                            ">>> {who} sent close with code {} and reason `{}`",
                            cf.code, cf.reason
                        );
                    } else {
                        println!(">>> {who} somehow sent close message without CloseFrame");
                    }
                    break;
                }
                _ => {}
            }
        } else {
            let error = msg.err().unwrap();
            println!("Error receiving message: {:?}", error);
            send_close_message(socket, 1011, &format!("Error occured: {}", error)).await;
            break;
        }
    }
}
