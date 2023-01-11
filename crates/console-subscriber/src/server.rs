use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc;
use tracing::Subscriber;
use tracing_subscriber::Layer;

use crate::{
    command::{Command, Watch},
    Aggregator,
};

use chekov_api as proto;

#[derive(Debug)]
pub struct Server {
    pub(crate) subscribe: mpsc::Sender<Command>,
    pub(crate) aggregator: Option<Aggregator>,
    pub addr: SocketAddr,
    pub client_buffer: usize,
}

impl Server {
    pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const DEFAULT_PORT: u16 = 6680;

    pub const DEFAULT_CLIENT_BUFFER_CAPACITY: usize = 1024 * 4;

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.serve_with(tonic::transport::Server::default()).await
    }

    pub async fn serve_with(
        mut self,
        mut builder: tonic::transport::Server,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let aggregate = self
            .aggregator
            .take()
            .expect("cannot start server multiple times");
        let aggregate = tokio::spawn(aggregate.run());

        let addr = self.addr;
        let serve = builder
            .add_service(proto::ChekovServer::new(self))
            .serve(addr);
        let res = tokio::spawn(serve).await;
        // aggregate.abort();
        res?.map_err(Into::into)
    }
}

#[tonic::async_trait]
impl proto::ChekovAPI for Server {
    type WatchUpdatesStream =
        tokio_stream::wrappers::ReceiverStream<Result<proto::Update, tonic::Status>>;

    async fn watch_updates(
        &self,
        request: tonic::Request<proto::client::WatchUpdateRequest>,
    ) -> Result<tonic::Response<Self::WatchUpdatesStream>, tonic::Status> {
        match request.remote_addr() {
            Some(addr) => tracing::debug!(client.addr = %addr, "starting a new watch"),
            None => tracing::debug!(client.addr = %"<unknown>", "starting a new watch"),
        }

        println!("request: {:?}", request.into_inner());
        let permit = self.subscribe.reserve().await.map_err(|_| {
            tonic::Status::internal("cannot start new watch, aggregation task is not running")
        })?;

        let (tx, rx) = mpsc::channel(self.client_buffer);
        permit.send(Command::Update(Watch(tx)));
        tracing::debug!("watch started");

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        //
        // let s = proto::streams::StreamUpdate {};
        //
        // let ss: Vec<u8> = serialize(&s);
        //
        // println!("{:?}", stream);
        Ok(tonic::Response::new(stream))
    }
}

fn serialize<T>(message: &T) -> Vec<u8>
where
    T: prost::Message,
{
    message.encode_to_vec()
}
