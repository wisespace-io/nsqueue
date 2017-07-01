use futures::{Future, future};

use tokio_service::Service;
use tokio_core::reactor::Handle;

use tokio_proto::{TcpClient};
use tokio_proto::util::client_proxy::ClientProxy;
use tokio_proto::streaming::{Message};

use std::io;
use std::net::SocketAddr;

use config::Config;
use response::{NSQ, ResponseStream};
use codec::{NsqMessage, NsqResponseMessage, ClientTypeMap};
use protocol::{NsqProtocol, RequestMessage};

#[derive(Clone)]
pub struct Consumer {
    inner: ClientTypeMap<ClientProxy<NsqMessage, NsqResponseMessage, io::Error>>,
}

impl Consumer {
    /// Establish a connection and send protocol version.
    pub fn connect(addr: &SocketAddr, handle: &Handle, config: Config) -> Box<Future<Item = Consumer, Error = io::Error>> {
        let protocol = NsqProtocol::new(config);
        let ret = TcpClient::new(protocol)
            .connect(addr, handle)
            .map(|client_proxy| {
                let type_map = ClientTypeMap { inner: client_proxy };
                Consumer { inner: type_map }
            });

        Box::new(ret)
    } 

    #[allow(unused_variables)]
    pub fn subscribe(&self, topic: String, channel: String) -> Box<Future<Item = NSQ, Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_sub_command(topic, channel);        
        
        let service = self.inner.clone();

        let resp = service.inner.call(Message::WithoutBody(request))
            .map_err(|e| {e.into()})
            .and_then(move |resp| {
                let mut request = RequestMessage::new();
                request.create_rdy_command();
                let rdy = service.inner.call(Message::WithoutBody(request))
                    .map_err(|e| {e.into()});
                rdy    
            })
            .map(move |resp| {                                  
                match resp {
                    Message::WithoutBody(str) => {
                        if str == "_heartbeat_" {
                            // Need to handle the heartbeat
                        }

                        panic!("Not supported: {}", str)
                    },
                    Message::WithBody(head, body) => {       
                        println!("many");           
                        NSQ::Stream(ResponseStream { inner: body })
                    }
                }
            });

        Box::new(resp)
    } 

    #[allow(unused_variables)]
    pub fn fin(&self, message_id: String) -> Box<Future<Item = (), Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_fin_command(message_id);        
        
        let service = self.inner.clone();
        let resp = service.inner.call(Message::WithoutBody(request))
            .map_err(|e| e.into())
            .and_then(|resp| future::ok(()));

        Box::new(resp)
    }    

    #[allow(unused_variables)]
    pub fn nop(&self) -> Box<Future<Item = (), Error = io::Error>> {
        let mut request = RequestMessage::new();
        request.create_nop_command();        
        
        let service = self.inner.clone();
        let resp = service.inner.call(Message::WithoutBody(request))
            .map_err(|e| e.into())
            .and_then(|resp| future::ok(()));

        Box::new(resp)
    }    
}

impl<T> Service for ClientTypeMap<T>
    where T: Service<Request = RequestMessage, Response = NsqResponseMessage, Error = io::Error>,
          T::Future: 'static
{
    type Request = RequestMessage;
    type Response = NsqResponseMessage;
    type Error = io::Error;
    type Future = Box<Future<Item = NsqResponseMessage, Error = io::Error>>;

    fn call(&self, req: RequestMessage) -> Self::Future {
        Box::new(self.inner.call(req))
    }
}