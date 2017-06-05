use std::io;
use futures::{Stream, Poll};
use tokio_proto::streaming::{Body};

#[derive(Debug)]
pub struct ResponseStream {
    inner: Body<String, io::Error>,
}

impl Stream for ResponseStream {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        self.inner.poll()
    }
}