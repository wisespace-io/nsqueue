extern crate futures;
extern crate tokio_core;
extern crate nsqueue;

use futures::Future;
use tokio_core::reactor::Core;

use nsqueue::config::*;
use nsqueue::producer::*;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:4150".parse().unwrap();

    let mut messages: Vec<String> = Vec::new();
    messages.push("First message".into());
    messages.push("Second message".into());

    let res = Producer::connect(&addr, &handle, Config::default())
       .and_then(|conn| {
           conn.mpublish("some_topic".into(), messages)
           .and_then(move |response| {
              println!("Response: {:?}", response);
              Ok(())
           })
       });
    core.run(res).unwrap();
}