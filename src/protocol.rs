use std::io;

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Framed};
use tokio_proto::pipeline::{ClientProto};

use serde_json::{to_string};

use futures::{Future, Stream, Sink};

use commands::*;
use codec::NsqCodec;
use config::Config;

/// Protocol definition
pub struct NsqProtocol {
    pub config: Config,
}

impl NsqProtocol {
    pub fn new(config: Config) -> Self {
        NsqProtocol {
            config: config,
        }
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for NsqProtocol {
    type Request = RequestMessage;
    type Response = String;
    type Transport = Framed<T, NsqCodec>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let config = self.config.clone();
        let mut request = RequestMessage::new();
        request.set_protocol_version(commands::VERSION_2);
                
        // Send protocol version
        let handshake = io.framed(NsqCodec).send(request.clone())
        .and_then(move |transport| {
            let mut request = RequestMessage::new();
            request.create_identify_command(config);
            
            // Send IDENTIFY
            let ch = transport.send(request.clone())
                .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
                .and_then(|(resp, transport)| {
                    Ok(transport)                      
                });

            ch
        });
        
        Box::new(handshake)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct RequestMessage {
    pub version: Option<String>, 
    pub header: Option<String>,
    pub body: Option<String>,
    pub body_messages: Option<Vec<String>>,
}

impl RequestMessage {
    pub fn new() -> RequestMessage {
        RequestMessage {
            version: None,
            header: None,
            body: None,
            body_messages: None,
        }
    }

    pub fn set_protocol_version(&mut self, version: &str) {
        self.version = Some(String::from(version));
    }

    pub fn create_pub_command(&mut self, topic: String, message: String) {
        self.header = Some(format!("{} {}\n", commands::PUB, topic));
        self.body = Some(message);
    }

    pub fn create_mpub_command(&mut self, topic: String, messages: Vec<String>) {
        self.header = Some(format!("{} {}\n", commands::MPUB, topic));
        self.body_messages = Some(messages);
    }

    pub fn create_dpub_command(&mut self, topic: String, message: String, defer_time: i64) {
        self.header = Some(format!("{} {} {}\n", commands::DPUB, topic, defer_time.to_string()));
        self.body = Some(message);
    }

    pub fn create_sub_command(&mut self, topic: String, channel: String) {
        self.header = Some(format!("{} {} {}\n", commands::SUB, topic, channel));
    }

    pub fn create_identify_command(&mut self, config: Config) {
        self.header = Some(format!("{}\n", commands::IDENTIFY));
        // Serialize it to a JSON string.
        self.body = Some(to_string(&config).unwrap());
    }  

    pub fn create_rdy_command(&mut self) {
        self.header = Some(format!("{} 1\n", commands::RDY));
    }       
}