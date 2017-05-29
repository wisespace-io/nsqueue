use futures::{Future, Poll, Stream, IntoFuture};
use futures::future::Either;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::{TcpClient};
use tokio_proto::pipeline::{ClientService};
use tokio_service::{Service};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Framed};

use std::io;
use std::net::SocketAddr;

use config::Config;
use codec::NsqCodec;
use protocol::{NsqProtocol, RequestMessage};

#[derive(Clone)]
pub struct Consumer {
    inner: ClientService<TcpStream, NsqProtocol>,
}

impl Consumer {
    /// Establish a connection and send protocol version.
    pub fn connect(addr: &SocketAddr, handle: &Handle, config: Config) -> Box<Future<Item = Consumer, Error = io::Error>> {
        let ret = TcpClient::new(NsqProtocol::new(config))
            .connect(addr, handle)
            .map(|client_service| {
                Consumer { inner: client_service }
            });

        Box::new(ret)
    } 

    // TODO: Box<Future<Item = Subscriber<T>, Error = io::Error>>
    pub fn subscribe(&self, topic: String, channel: String) -> Box<Future<Item = String, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_sub_command(topic, channel);        
        
        let service = self.inner.clone();

        let resp = self.call(request)
            .map_err(|e| {e.into()})
            .and_then(move |resp| {
                if resp != "OK" {
                    let fail: Result<String, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "Failed to subscribe to a channel"));
                    Either::A(fail.into_future())
                } else {
                    // Update RDY state (should use --max-rdy-count to bound this value)
                    let mut request = RequestMessage::new();
                    request.create_rdy_command();
                    let rdy = service.call(request)
                        .map_err(|e| {e.into()});
                    Either::B(rdy)    
                }
            });

        Box::new(resp)
    } 
}

impl Service for Consumer {
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

// Implement as a stream
pub struct Subscriber<T> {
  pub transport: Framed<T, NsqCodec>,
}

impl<T: AsyncRead+AsyncWrite+'static> Stream for Subscriber<T> {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        self.transport.poll()
    }
}