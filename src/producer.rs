use futures::{Future};

use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::{TcpClient};
use tokio_proto::pipeline::{ClientService};
use tokio_service::{Service};

use std::io;
use std::net::SocketAddr;

use config::Config;
use protocol::{NsqProtocol, RequestMessage};

pub struct Producer {
    inner: ClientService<TcpStream, NsqProtocol>,
}

impl Producer {
    /// Establish a connection and send protocol version.
    pub fn connect(addr: &SocketAddr, handle: &Handle, config: Config) -> Box<Future<Item = Producer, Error = io::Error>> {
        let ret = TcpClient::new(NsqProtocol::new(config))
            .connect(addr, handle)
            .map(|client_service| {
                Producer { inner: client_service }
            });

        Box::new(ret)
    }

    pub fn publish(&self, topic: String, message: String) -> Box<Future<Item = String, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_pub_command(topic, message);        
        
        let resp = self.call(request)
            .map_err(|e| {
                 e.into()}
                )
            .and_then(|resp| {
                if resp != "OK" {
                    Err(io::Error::new(io::ErrorKind::Other, "expected OK"))
                } else {
                    Ok(resp)
                }
            });

        Box::new(resp)
    }

    pub fn mpublish(&self, topic: String, messages: Vec<String>) -> Box<Future<Item = String, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_mpub_command(topic, messages);        
        
        let resp = self.call(request)
            .map_err(|e| {
                 e.into()}
                )
            .and_then(|resp| {
                if resp != "OK" {
                    Err(io::Error::new(io::ErrorKind::Other, "expected OK"))
                } else {
                    Ok(resp)
                }
            });

        Box::new(resp)
    }    
}

impl Service for Producer {
    type Request = RequestMessage;
    type Response = String;
    type Error = io::Error;
    type Future = Box<Future<Item = String, Error = io::Error>>;

    fn call(&self, req: RequestMessage) -> Self::Future {
        Box::new(self.inner.call(req)
            .map_err(|e| {
                e.into()}
            )
            .and_then(|resp| {
                Ok(resp)
            }))
    }
}