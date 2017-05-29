use std::io;
use std::io::Cursor;
use bytes::{Buf, BufMut, BytesMut, BigEndian};
use tokio_io::codec::{Encoder, Decoder};
use std::str;

use protocol::{RequestMessage};

// Header: Size(4-Byte) + FrameType(4-Byte)
const HEADER_LENGTH: usize = 8;

// Frame Types
const FRAME_TYPE_RESPONSE: i32 = 0x00;
const FRAME_TYPE_ERROR: i32 =  0x01;
const FRAME_TYPE_MESSAGE: i32 =  0x02;

/// NSQ codec
pub struct NsqCodec;

impl Decoder for NsqCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let length = buf.len();

        if length < HEADER_LENGTH {
            return Ok(None);
        }

        let mut cursor = Cursor::new(buf.clone());
        let size : i32 = cursor.get_i32::<BigEndian>();

        if length < size as usize {
            return Ok(None);
        }

        let frame_type: i32 = cursor.get_i32::<BigEndian>();

        // remove the serialized frame from the buffer.
        buf.split_to(HEADER_LENGTH + length);

        if frame_type == FRAME_TYPE_RESPONSE {
            match str::from_utf8(&cursor.bytes()) {
                Ok(s) => {
                   // return the parsed response
                   Ok(Some(s.to_string()))
                },
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8")),
            }
        } else if frame_type == FRAME_TYPE_ERROR {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid packet received"))
        } else if frame_type == FRAME_TYPE_MESSAGE {
            let _ = cursor.get_i64::<BigEndian>(); // timestamp
            let _ = cursor.get_u16::<BigEndian>(); // attempts
            let _ = cursor.get_i16::<BigEndian>(); // message_id
         
            // message
            match str::from_utf8(&cursor.bytes()) {
                Ok(s) => {
                   // return the parsed response
                   Ok(Some(s.to_string()))
                },
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }    
}

impl Encoder for NsqCodec {
    type Item = RequestMessage;
    type Error = io::Error;

    fn encode(&mut self, message: RequestMessage, buf: &mut BytesMut) -> io::Result<()> {
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

        Ok(())
    }
}