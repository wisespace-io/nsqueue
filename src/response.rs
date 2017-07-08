use std::io;
use futures::{Stream, Poll, Async};
use tokio_proto::streaming::Body;

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

/*
impl<T> Sink for ResponseStream<T>
    where T: Sink<SinkItem = String, SinkError = io::Error>,
{
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, item: String) -> StartSend<String, io::Error> {
        self.upstream.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        self.upstream.poll_complete()
    }
}
*/

pub struct Message {
    pub timestamp: i64,
    pub message_id: String,
    pub message_body: String
}