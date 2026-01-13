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
    async fn on_start(state: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("WebClient Actor started");
        Ok(WebClient::new(state.id, state.hub, state.is_pod))
    }
}
impl Message<ClientResponse> for WebClient {
    type Reply = ClientResponse;

    async fn handle(
        &mut self,
        msg: ClientResponse,
        _ctx: &mut Context<Self, Self::Reply>,
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
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let ClientRequestAsync::RequestImage { client_id, .. } = &mut msg;
        *client_id = self.id;
        let _ = self.hub.tell(msg);
    }
}
impl Message<PodResponse> for WebClient{
    type Reply = ();
    // Todo implement MessageResult -PodResponse
    async fn handle(
        &mut self,
        msg: PodResponse,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let _msg = JsonProtocol::PodResponse(msg);
    }
}

impl Message<PodRequest> for WebClient{
    type Reply = ();
    async fn handle(
        &mut self,
        msg: PodRequest,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
                use PodRequest::*;
        match msg {
            RegisterSelf { name, .. } => {
                if !self.is_pod {
                    self.is_pod = true;
                    let _ = self.hub.tell(SubscribePod { id: self.id, name, addr: ctx.actor_ref().clone(), });
                    //actix::Handler::handle(self, PodResponse::Registered { global_id: self.id }, ctx);
                    ctx.forward(&ctx.actor_ref().clone(), PodResponse::Registered { global_id: self.id }).await;
                } else {
                    //actix::Handler::handle(self, PodResponse::AlreadyRegistered { global_id: self.id }, ctx);
                    ctx.forward(&ctx.actor_ref().clone(), PodResponse::AlreadyRegistered { global_id: self.id }).await.;
                }
            }
            other_messages => {
                let _ = self.hub.tell(IdedPodRequest { id: self.id, message: other_messages });
            }
        };
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
            let _ = addr.tell(message.clone());
        }
    }

}

impl Actor for Hub {
    type Args = Self;
    type Error = Infallible;
    async fn on_start(_state: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("HubActor started");
        Ok(Hub::new())
    }
}

impl Message<SubscribeClient> for Hub {
    type Reply = ();
    async fn handle(
        &mut self,
        msg: SubscribeClient,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clients.insert(msg.id, msg.addr);
        ctx.forward(&ctx.actor_ref().clone(), ClientRequest::ListAllPods).await;
        // maybe do self request through ctx.handle(...)
    }
}

impl Message<UnsubscribeClient> for Hub {
    type Reply = ();
    async fn handle(
        &mut self,
        msg: UnsubscribeClient,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clients.remove(&msg.0);

        // and pod if it was serving data
        if let Some(lost_pod) = self.pods.remove(&msg.0) {
            let message = ClientResponse::PodGone(msg.0);
            self.broadcast_client_response(message);
            println!("removing pod {}: {:?}", msg.0, lost_pod);
            println!("  > remaining: {:?}", self.pods);
        }

        println!("UnsubscribeClient: {:?}", msg.0);

    }
}

impl Message<SubscribePod> for Hub{
    type Reply = ();
    async fn handle(
        &mut self,
        msg: SubscribePod,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.pods.insert(msg.id, PodInfo {
            addr: msg.addr,
            name: msg.name.clone(),
            image_paths: vec![],
            last_modified: Utc::now(),
        });
        self.broadcast_client_response(ClientResponse::NewPod { id: msg.id, name: msg.name, });
    }
}

impl Message<UnsubscribePod> for Hub{
    type Reply = ();
    async fn handle(
        &mut self,
        msg: UnsubscribePod,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(lost_pod) = self.pods.remove(&msg.0) {
            let message = ClientResponse::PodGone(msg.0);
            self.broadcast_client_response(message);
            println!("removing pod {}:{:?}", msg.0, lost_pod.name);
        }
    }

}

impl Message<ClientRequest> for Hub {
    type Reply = ClientResponse;
    async fn handle(
        &mut self,
        msg: ClientRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        use crate::protocols::ClientRequest::*;
        let r = match msg {
            ListAllPods => {
                let pods = self.pods.iter().map(|(&id, info)| {
                    crate::protocols::PodDescription {
                        id,
                        name: info.name.clone(),
                        paths: info.image_paths.clone(),
                        last_modified: info.last_modified,
                    }
                }).collect();
                ClientResponse::Pods(pods)
            }
            ListPodStructure(id) => {
                match self.pods.get(&id) {
                    Some(info) => {
                        ClientResponse::PodUpdatePaths {
                            id,
                            paths: info.image_paths.clone(),
                            replace_images: false,
                            last_modified: info.last_modified,
                        }
                    }
                    None => {
                        ClientResponse::UnknownPod(id)
                    }
                }
            }
        };
        r
    }
}

impl Message<ClientRequestAsync> for Hub {
    type Reply = ();
    async fn handle(
        &mut self,
        msg: ClientRequestAsync,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            ClientRequestAsync::RequestImage { gallery_id, path, client_id } => {
                match self.pods.get(&gallery_id) {
                    Some(pod) => {
                        let _ = pod.addr.tell(PodResponse::RequestImage{
                            path,
                            client_id
                        });
                    }
                    None => {
                        if let Some(client) = self.clients.get(&client_id) {
                            let _ = client.tell(
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
        _ctx: &mut Context<Self, Self::Reply>,
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
                    let _ = client.tell(ClientResponse::DeliverImage {
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
