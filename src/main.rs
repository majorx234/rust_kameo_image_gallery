use std::{net::SocketAddr, path::PathBuf, sync::{Arc, Mutex, atomic::AtomicU64}};

use kameo::prelude::*;
use infra::{actors::{self, Hub, WebClient}, config::Config, webserver::{AppState, Incrementor, websocket_handler}};
use axum::{extract::State, Router, routing::any};
use tower_http::{services::ServeDir,trace::{DefaultMakeSpan, TraceLayer}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start only one instance of our central Hub
    let hub = Hub::spawn(Hub::default());

    let web_state = AppState{
        incrementor: std::sync::Arc::new(Mutex::new(Incrementor::new())),
        //next_id: std::sync::Arc::new(AtomicU64::new(1)),
        actor_ref: hub.clone(),
    };
    println!("Hub created!");

    let config = Config::new();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(config.get_rust_log()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from("./static/");
    let app = Router::new().fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", any(websocket_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        ).with_state(web_state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    println!("ws-Webserver created!");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    hub.wait_for_shutdown().await;
    Ok(())
}
