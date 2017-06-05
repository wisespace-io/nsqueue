extern crate futures;
extern crate tokio_core;
extern crate nsqueue;

use futures::Future;
use tokio_core::reactor::Core;

use nsqueue::config::*;
use nsqueue::consumer::*;

fn main() {
     let mut core = Core::new().unwrap();
     let handle = core.handle();

     let addr = "127.0.0.1:4150".parse().unwrap();

     core.run(
         Consumer::connect(&addr, &handle, Config::default())
         .and_then(|conn| {
            // TODO: Implement subscription as a stream 
            conn.subscribe("some_topic".into(), "some_channel".into())
            .and_then(|message| {
                println!("Got message {}", message);
                Ok(())
             })
         })
     ).unwrap();
}