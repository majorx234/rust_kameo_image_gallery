use serde_json as json;
use serde_derive::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

pub type PodId = u64; // <- danger zone

/// Master -> Browser
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientResponse {
    Pods(Vec<PodDescription>),
    NewPod { id: PodId, name: String, },
    UnknownPod(PodId),
    PodGone(PodId),
    PodUpdateName { id: PodId, name: String, },
    PodUpdatePaths { id: PodId, paths: Vec<String>, replace_images: bool, last_modified: DateTime<Utc>, },
    DeliverImage { gallery_id: PodId, path: String, blob: String, },
}

/// Browser -> Master rpc style
#[derive(Serialize, Deserialize, Debug)]
//#[rtype(result = "ClientResponse")]
pub enum ClientRequest{
    ListAllPods,
    ListPodStructure(PodId),
}

/// Browser -> Master
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequestAsync {
    RequestImage {
        gallery_id: PodId,
        path: String,
        #[serde(skip)]
        client_id: PodId,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PodDescription {
    pub id: PodId,
    pub name: String,
    pub paths: Vec<String>,
    pub last_modified: DateTime<Utc>,
}

/// Slave -> Master
#[derive(Serialize, Deserialize, Debug)]
pub enum PodRequest {
    RegisterSelf {
        proposed_id: Option<PodId>,
        name: String,
    },
    UpdateTitle { name: String, },
    UpdatePaths { paths: Vec<String>, replace_images: bool, },
    DeliverImage { client_id: PodId, path: String, blob: String, },
}
/// Master -> Slave
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PodResponse {
    Registered { global_id: PodId, },
    AlreadyRegistered { global_id: PodId, },
    RequestImage { client_id: PodId, path: String },
}

/// Communicate with everything
#[derive(Serialize, Deserialize, Debug)]
pub enum JsonProtocol {
    ClientRequest(ClientRequest),
    ClientRequestAsync(ClientRequestAsync),
    ClientResponse(ClientResponse),
    PodRequest(PodRequest),
    PodResponse(PodResponse),
}

// kind of testfunction
pub(crate) fn print_all_messages() {
    let t = |t| { println!("\n==== {} ====", t); };
    let p = |obj| {
        let s = json::to_string(&obj).unwrap();
        println!("  {}", s);
    };
    let last_modified = Utc::now();

    t("ClientRequest");
    p(JsonProtocol::ClientRequest(ClientRequest::ListAllPods));
    p(JsonProtocol::ClientRequest(ClientRequest::ListPodStructure(42)));

    t("ClientRequestAsync");
    p(JsonProtocol::ClientRequestAsync(ClientRequestAsync::RequestImage{gallery_id:42, path: "bla".into(), client_id: 0, }));

    t("ClientResponse");
    p(JsonProtocol::ClientResponse(ClientResponse::Pods(
        vec![PodDescription{id: 42, name: "bla".into(), paths: vec![], last_modified,}])));
    p(JsonProtocol::ClientResponse(ClientResponse::NewPod{id:23, name: "blubb".into()}));
    p(JsonProtocol::ClientResponse(ClientResponse::UnknownPod(123)));
    p(JsonProtocol::ClientResponse(ClientResponse::PodGone(1234)));
    p(JsonProtocol::ClientResponse(ClientResponse::PodUpdateName{ id: 42, name: "String".into(), }));
    p(JsonProtocol::ClientResponse(ClientResponse::PodUpdatePaths{ id: 42, paths: vec!["String".into()], replace_images: false, last_modified, }));
    p(JsonProtocol::ClientResponse(ClientResponse::DeliverImage { gallery_id: 42, path: "String".into(), blob: "String".into(), },));


    t("PodRequest");
    p(JsonProtocol::PodRequest(PodRequest::RegisterSelf{ proposed_id: Some(42), name: "bla".into(), }));
    p(JsonProtocol::PodRequest(PodRequest::RegisterSelf{ proposed_id: None, name: "bla".into(), }));
    p(JsonProtocol::PodRequest(PodRequest::UpdateTitle{ name: "bli".into(), }));
    p(JsonProtocol::PodRequest(PodRequest::UpdatePaths{ paths: vec!["bli".into()], replace_images: true, }));
    p(JsonProtocol::PodRequest(PodRequest::DeliverImage { client_id: 23, path: "String".into(), blob: "String".into(), },));

    t("PodResponse");
    p(JsonProtocol::PodResponse(PodResponse::Registered { global_id: 42, }));
    p(JsonProtocol::PodResponse(PodResponse::AlreadyRegistered { global_id: 42, }));
    p(JsonProtocol::PodResponse(PodResponse::RequestImage { client_id: 42, path: "bli".into(), }));

    println!("\n");
}
