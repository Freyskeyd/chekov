use actix::Message;
use event_store_core::error::EventStoreError;
use event_store_core::stream::Stream as EventStoreStream;

pub mod aggregates;
pub mod client;
pub mod events;
pub mod streams;

pub use client::chekov_client::ChekovClient;
pub use client::chekov_server::ChekovServer;
pub use client::Update;

#[derive(Debug)]
pub struct StreamServer {}

impl StreamServer {
    // pub fn new(get_stream_list: Recipient<GetStreamList>) -> Self {
    //     Self { get_stream_list }
    // }
}
//
// #[tonic::async_trait]
// impl Stream for StreamServer {
//     async fn get_stream_list(
//         &self,
//         request: tonic::Request<GetStreamListRequest>,
//     ) -> Result<tonic::Response<GetStreamListResponse>, tonic::Status> {
//         if let Ok(Ok(streams)) = self.get_stream_list.send(GetStreamList {}).await {
//             let response = GetStreamListResponse {
//                 streams: streams.into_iter().map(|s| s.stream_uuid).collect(),
//             };
//             Ok(Response::new(response))
//         } else {
//             Err(tonic::Status::unimplemented("soon"))
//         }
//     }
// }
//
pub struct Server {}
impl Server {
    pub async fn start(
        _event_store: actix::Recipient<GetStreamList>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = "[::1]:50051".parse()?;

        // let stream = StreamServer::new(event_store);

        println!("Starting gRPC Server...");
        tonic::transport::Server::builder()
            // .add_service(streams::stream_server::StreamServer::new(stream))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Vec<EventStoreStream>, EventStoreError>")]
pub struct GetStreamList {}
