use std::collections::HashMap;

use kameo::{error::Infallible, prelude::*};
use ::chrono::{Utc, DateTime};

use crate::protocols::{ClientRequest, ClientRequestAsync, ClientResponse, JsonProtocol, PodId, PodRequest, PodResponse};

pub struct WebClient {
    pub id: PodId,
    pub hub: ActorRef<Hub>,
    pub is_pod: bool,
}
impl WebClient {
    fn new(id: PodId, hub:ActorRef<Hub>,is_pod: bool) -> Self {
        WebClient{
            id,
            hub,
            is_pod,
        }
    }
}
impl Actor for WebClient {
    type Args = Self;
    type Error = Infallible;
    async fn on_start(state: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("WebClient Actor started");
        Ok(WebClient::new(state.id, state.hub, state.is_pod))
    }
}
impl Message<ClientResponse> for WebClient {
    type Reply = ();
    // todo need some kind of ClientResponse

    async fn handle(
        &mut self,
        msg: ClientResponse,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let message = JsonProtocol::ClientResponse(msg);
        // TODO: some wbsocket text stuff here
        // ctx.text(serde_json::to_string(&message).expect("unable to serialize internal state"));
    }
}


pub struct PodInfo {
    addr: ActorRef<WebClient>,
    name: String,
    image_paths: Vec<String>,
    last_modified: DateTime<Utc>,
}
impl std::fmt::Debug for PodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PodInfo")
            .field("name", &self.name)
            .field("image_paths", &self.image_paths)
            .finish()
    }
}

pub struct Hub {
    pods: HashMap<PodId, PodInfo>,
    clients: HashMap<PodId, ActorRef<WebClient>>,
}
impl Hub{
    fn new() -> Self {
        Hub{
            pods: HashMap::new(),
            clients: HashMap::new()
        }
    }
    fn broadcast_client_response(&self, message: ClientResponse) {
        for addr in self.clients.values() {
            addr.tell(message.clone());
        }
    }

}

impl Actor for Hub {
    type Args = Self;
    type Error = Infallible;
    async fn on_start(state: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("HubActor started");
        Ok(Hub::new())
    }
}
