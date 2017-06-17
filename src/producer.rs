use futures::{Future};

use tokio_service::Service;
use tokio_core::reactor::Handle;

use tokio_proto::{TcpClient};
use tokio_proto::streaming::{Message};
use tokio_proto::util::client_proxy::ClientProxy;

use std::io;
use std::net::SocketAddr;

use config::Config;
use codec::{NsqMessage, NsqResponseMessage, ClientTypeMap};
use protocol::{NsqProtocol, RequestMessage};

pub struct Producer {
    inner: ClientTypeMap<ClientProxy<NsqMessage, NsqResponseMessage, io::Error>>,
}

impl Producer {
    /// Establish a connection and send protocol version.
    pub fn connect(addr: &SocketAddr, handle: &Handle, config: Config) -> Box<Future<Item = Producer, Error = io::Error>> {
        let protocol = NsqProtocol::new(config);
        let ret = TcpClient::new(protocol)
            .connect(addr, handle)
            .map(|client_proxy| {
                let type_map = ClientTypeMap { inner: client_proxy };
                Producer { inner: type_map }
            });

        Box::new(ret)
    }

    // Publish a message to a topic
    pub fn publish(&self, topic: String, message: String) -> Box<Future<Item = NsqResponseMessage, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_pub_command(topic, message);        
        
        self.handler(request)
    }

    // Publish multiple messages to a topic (atomically)
    pub fn mpublish(&self, topic: String, messages: Vec<String>) -> Box<Future<Item = NsqResponseMessage, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_mpub_command(topic, messages);        
        
        self.handler(request)
    } 

    // Publish a deferred message to a topic
    pub fn dpublish(&self, topic: String, message: String, defer_time: i64) -> Box<Future<Item = NsqResponseMessage, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_dpub_command(topic, message, defer_time);
        
        self.handler(request)
    }

    fn handler(&self, request: RequestMessage) -> Box<Future<Item = NsqResponseMessage, Error = io::Error>> {
        let service = self.inner.clone();
        let resp = service.inner.call(Message::WithoutBody(request))
            .map_err(|e| {
                 e.into()}
                )
            .and_then(|resp| {
                if resp != "OK".into() {
                    Err(io::Error::new(io::ErrorKind::Other, "expected OK"))
                } else {
                    Ok(resp)
                }
            });

        Box::new(resp)        
    }
}