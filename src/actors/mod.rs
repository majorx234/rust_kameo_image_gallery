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
    type Reply = ClientResponse;

    async fn handle(
        &mut self,
        msg: ClientResponse,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        //let message = JsonProtocol::ClientResponse(msg);
        // TODO: some wbsocket text stuff here
        // ctx.text(serde_json::to_string(&message).expect("unable to serialize internal state"));
        msg.clone()
    }
}
impl Message<ClientRequestAsync> for WebClient{
    type Reply = ();
    async fn handle(
        &mut self,
        mut msg: ClientRequestAsync,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let ClientRequestAsync::RequestImage { client_id, .. } = &mut msg {
            *client_id = self.id;
        };
        self.hub.tell(msg);
    }
}
impl Message<PodResponse> for WebClient{
    type Reply = ();
    // Todo implement MessageResult -PodResponse
    async fn handle(
        &mut self,
        msg: PodResponse,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let msg = JsonProtocol::PodResponse(msg);
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
impl Message<ClientRequestAsync> for Hub {
    type Reply = ();
    async fn handle(
        &mut self,
        msg: ClientRequestAsync,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            ClientRequestAsync::RequestImage { gallery_id, path, client_id } => {
                match self.pods.get(&gallery_id) {
                    Some(pod) => {
                        pod.addr.tell(PodResponse::RequestImage{
                            path,
                            client_id
                        });
                    }
                    None => {
                        if let Some(client) = self.clients.get(&client_id) {
                            client.tell(
                                ClientResponse::UnknownPod(gallery_id)
                            );
                        }
                    }
                }
            }
        }
    }
}

impl Message<IdedPodRequest> for Hub {
    type Reply = ();
    async fn handle(
        &mut self,
        msg: IdedPodRequest,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        use crate::protocols::PodRequest::*;
        match msg.message {
            RegisterSelf { proposed_id, name } => unreachable!("must be handles by WebClient"),
            UpdateTitle { name } => {
                self.pods.get_mut(&msg.id).expect("unable to find PodInfo").name = name.clone();
                self.broadcast_client_response(ClientResponse::PodUpdateName{ id: msg.id, name, });
            }
            UpdatePaths { mut paths, replace_images } => {
                paths.sort();
                paths.dedup_by(|a, b| a == b);
                let now = Utc::now();
                let pod_info = self.pods.get_mut(&msg.id).expect("unable to find PodInfo");
                pod_info.image_paths = paths.clone();
                pod_info.last_modified = now;
                self.broadcast_client_response(ClientResponse::PodUpdatePaths{
                    id: msg.id,
                    paths,
                    replace_images,
                    last_modified: now,
                });
            }
            DeliverImage { client_id, path, blob } => {
                if let Some(client) = self.clients.get(&client_id) {
                    client.tell(ClientResponse::DeliverImage {
                        gallery_id: msg.id,
                        path,
                        blob,
                    });
                }
            }
        }
    }
}

pub struct SubscribePod {
    id: PodId,
    addr: ActorRef<WebClient>,
    name: String,
}

pub struct UnsubscribePod(PodId);

pub struct SubscribeClient {
    id: PodId,
    addr: ActorRef<WebClient>,
}

pub struct UnsubscribeClient(PodId);

pub struct IdedPodRequest {
    id: PodId,
    message: PodRequest,
}
