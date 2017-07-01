use std::io;
use std::io::Cursor;
use std::iter::Iterator;

use bytes::{Buf, BufMut, BytesMut, BigEndian};
use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::streaming::pipeline::Frame;
use tokio_proto::streaming::{Body, Message};
use std::str;

use protocol::RequestMessage;
use response::Message as TypeMessage;

// Header: Size(4-Byte) + FrameType(4-Byte)
const HEADER_LENGTH: usize = 8;

// Frame Types
const FRAME_TYPE_RESPONSE: i32 = 0x00;
const FRAME_TYPE_ERROR: i32 = 0x01;
const FRAME_TYPE_MESSAGE: i32 = 0x02;


const HEARTBEAT: &'static str = "_heartbeat_";

#[derive(Clone)]
pub struct ClientTypeMap<T> {
   pub inner: T,
}

pub type NsqMessage = Message<RequestMessage, Body<RequestMessage, io::Error>>;
pub type NsqResponseMessage = Message<String, Body<TypeMessage, io::Error>>;

/// NSQ codec
pub struct NsqCodec {
    pub decoding_head: bool
}

impl Decoder for NsqCodec {
    type Item = Frame<String, TypeMessage, io::Error>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let length = buf.len();

        if length < HEADER_LENGTH {
            return Ok(None);
        }

        let mut cursor = Cursor::new(buf.clone());
        let size: i32 = cursor.get_i32::<BigEndian>();

        if length < size as usize {
            return Ok(None);
        }

        let frame_type: i32 = cursor.get_i32::<BigEndian>();

        if frame_type == FRAME_TYPE_RESPONSE {
            // remove the serialized frame from the buffer.
            buf.split_to(HEADER_LENGTH + length);
            match str::from_utf8(&cursor.bytes()) {
                Ok(s) => {
                    let decoded_message = s.to_string();

                    // TODO: Implement a proper way to handle the heartbeat
                    if decoded_message == HEARTBEAT && !self.decoding_head {
                        Ok(Some(self.heartbeat_message()))
                    } else if decoded_message == HEARTBEAT {
                        // toggle streaming
                        Ok(Some(self.streaming_flag()))
                    } else {
                        Ok(Some(Frame::Message {
                            message: decoded_message,
                            body: false,
                        }))
                    }
                }
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8")),
            }
        } else if frame_type == FRAME_TYPE_ERROR {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid packet received"))
        } else if frame_type == FRAME_TYPE_MESSAGE {
            if self.decoding_head {
                // toggle streaming
                Ok(Some(self.streaming_flag()))
            } else {
                let timestamp = cursor.get_i64::<BigEndian>(); // timestamp
                let _ = cursor.get_u16::<BigEndian>(); // attempts
 
                let data = str::from_utf8(&cursor.bytes()).unwrap().to_string();
                let (id, body) = data.split_at(16);

                let message = TypeMessage{
                    timestamp: timestamp,
                    message_id: id.to_string(),
                    message_body: body.to_string()
                };

                // remove the serialized frame from the buffer.
                buf.split_to(HEADER_LENGTH + length);

                Ok(Some(
                    Frame::Body {
                        chunk: Some(message),
                    }
                ))              
            }
        } else {
            Ok(None)
        }
    }
}

pub type CodecOutputFrame = Frame<RequestMessage, RequestMessage, io::Error>;
impl Encoder for NsqCodec {
    type Item = CodecOutputFrame;
    type Error = io::Error;

    fn encode(&mut self, message: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        match message {
            Frame::Message { message, .. } => {
                if let Some(version) = message.version {
                    buf.reserve(version.len());
                    buf.extend(version.as_bytes());
                }

                if let Some(header) = message.header {
                    buf.reserve(header.len() + 1);
                    buf.extend(header.as_bytes());
                }

                if let Some(body) = message.body {
                    let mut buf_32 = Vec::with_capacity(body.len());
                    let body_len = body.len() as u32;
                    buf_32.put_u32::<BigEndian>(body_len);
                    buf_32.put(&body[..]);
                    buf.extend(buf_32);
                }

                if let Some(body_messages) = message.body_messages {
                    let total_bytes = body_messages
                        .iter()
                        .map(|message| message.len())
                        .fold(0, |acc, len| acc + len);

                    let mut buf_32 = Vec::with_capacity(total_bytes);
                    // [4-byte body size]
                    let body_len = total_bytes as u32;
                    buf_32.put_u32::<BigEndian>(body_len);
                    // [4-byte num messages]
                    let messages_len = body_messages.len() as u32;
                    buf_32.put_u32::<BigEndian>(messages_len);
                    // [ 4-byte message #1 size ][ N-byte binary data ] ...
                    for message in &body_messages {
                        let message_len = message.len() as u32;
                        buf_32.put_u32::<BigEndian>(message_len);
                        buf_32.put(&message[..]);
                    }
                    buf.extend(buf_32);
                }
                Ok(())
            }
            Frame::Error { error, .. } => Err(error),
            Frame::Body { .. } => panic!("Streaming of Requests is not currently supported"),
        }
    }
}

impl NsqCodec {
    
    fn heartbeat_message(&mut self) -> Frame<String, TypeMessage, io::Error>
    {
        let message = TypeMessage{
            timestamp: 0,
            message_id: HEARTBEAT.to_string(),
            message_body: HEARTBEAT.to_string()
        };

        Frame::Body {
            chunk: Some(message),
        } 
    }

    fn streaming_flag(&mut self) -> Frame<String, TypeMessage, io::Error>
    {
        self.decoding_head = false;
        Frame::Message {
            message: "".into(),
            body: true,
        }
    }
}