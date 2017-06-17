use std::io;
use futures::{Stream, Poll};
use tokio_proto::streaming::Body;

#[derive(Debug)]
pub enum NSQ {
    Stream(ResponseStream),
}

#[derive(Debug)]
pub struct ResponseStream {
    pub header: String,
    pub inner: Body<String, io::Error>,
}

impl Stream for ResponseStream {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        // It seems that I need to handle body response here
        self.inner.poll()
    }
}