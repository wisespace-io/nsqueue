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

    let res = Producer::connect(&addr, &handle, Config::default())
       .and_then(|conn| {
           conn.publish("some_topic".into(), "some_message".into())
           .and_then(move |response| {
              println!("Response: {:?}", response);
              Ok(())
           })
       });
    core.run(res).unwrap();
}