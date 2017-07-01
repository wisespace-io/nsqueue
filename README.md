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
$ nsqlookupd & 
$ nsqd --lookupd-tcp-address=127.0.0.1:4160 &
$ nsqadmin --lookupd-http-address=127.0.0.1:4161 &
```

### PUB
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
                        let ret = response.for_each(move |message| {
                            println!("Response: {:?}", message.body);
                            conn.fin(message.message_id); // Inform NSQ (Message consumed)
                            Ok(()) 
                        });
                        ret
                    }
                }
            })
         })
     ).unwrap();
}
```

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)
