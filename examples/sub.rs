extern crate futures;
extern crate tokio_core;
extern crate nsqueue;

use futures::{Stream, Future};
use tokio_core::reactor::Core;

use nsqueue::config::*;
use nsqueue::consumer::*;
use nsqueue::response::NSQ;

fn main() {
     let mut core = Core::new().unwrap();
     let handle = core.handle();

     let addr = "127.0.0.1:4150".parse().unwrap();

     core.run(
         Consumer::connect(&addr, &handle, Config::default())
         .and_then(|conn| {
            conn.subscribe("some_topic".into(), "some_channel".into())
            .and_then(move |message| {
                match message {
                    NSQ::Stream(response) => {
                        let message_id = response.header.clone();
                        let ret = response.for_each(move |message| {
                            println!("Response: {:?}", message);
                            conn.fin(message_id.clone()) // Inform NSQ (Message consumed) 
                        });
                        ret
                    }
                }
            })
         })
     ).unwrap();
}