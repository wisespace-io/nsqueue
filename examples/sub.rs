extern crate futures;
extern crate tokio_core;
extern crate nsqueue;

use futures::{Stream, Future};
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
            conn.subscribe("some_topic".into(), "some_channel".into())
            .and_then(move |response| {
                let ret = response.for_each(move |message| {
                    if message.message_id == "_heartbeat_" {
                        conn.nop();
                    } else {
                        println!("Response {:?} {:?}", message.message_id, message.message_body);
                        conn.fin(message.message_id); // Inform NSQ (Message consumed)
                    }
                    Ok(())
                });
                ret
            })
         })
     ).unwrap();
}