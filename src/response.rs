use std::io;
use futures::{Stream, Poll, Async};
use tokio_proto::streaming::Body;

#[derive(Debug)]
pub enum NSQ {
    Stream(ResponseStream),
}

#[derive(Debug)]
pub struct ResponseStream {
    pub inner: Body<Message, io::Error>,
}

impl Stream for ResponseStream {
    type Item = Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Message>, io::Error> {
        match self.inner.poll().unwrap() {
            Async::Ready(Some(request)) => {
                Ok(Async::Ready(Some(request)))
            }
            Async::Ready(None) => {
                // the stream finished.
                Ok(Async::Ready(None))
            }
            Async::NotReady =>  {
                // no more messages to read
                Ok(Async::NotReady)
            }
        }        
    }
}

pub struct Message {
    pub timestamp: i64,
    pub message_id: String,
    pub message_body: String
}