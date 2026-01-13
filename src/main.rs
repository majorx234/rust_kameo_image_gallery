use kameo::prelude::*;
use infra::actors::{self, Hub};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor_ref = Hub::spawn(Hub::default());
    println!("Hub created!");

    actor_ref.wait_for_shutdown().await;
    Ok(())
}
