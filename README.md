[![Build Status](https://travis-ci.org/wisespace-io/nsqueue.png?branch=master)](https://travis-ci.org/wisespace-io/nsqueue)
[![Crates.io](https://img.shields.io/crates/v/nsqueue.svg)](https://crates.io/crates/nsqueue)

# nsqueue
A [Tokio](https://tokio.rs/) based client implementation for the [NSQ](https://github.com/bitly/nsq) realtime message processing system

## WORK IN PROGRESS

### Current features
- [X] PUB
- [X] SUB
- [ ] Discovery
- [ ] Backoff 
- [ ] TLS
- [ ] Snappy
- [ ] Auth

### Launch NSQ
```
$ ./nsqlookupd & 
$ ./nsqd --lookupd-tcp-address=127.0.0.1:4160 &
$ ./nsqadmin --lookupd-http-address=127.0.0.1:4161 &
```

### MPUB
```
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
```

### SUB
```
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
```

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)
