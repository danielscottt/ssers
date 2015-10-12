ssers
=====

## Example:

```rust
#[macro_use]
extern crate log;
extern crate ssers;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

use ssers::SSE;

fn main() {
    let (tx,rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let sse = SSE::new(String::from("http://sse/events"), false);
    thread::spawn(move || {
        match sse.listen(tx) {
            Ok(_)  => (),
            Err(e) => {
                warn!("{:?}", e);
                return
            },
        };
    });

    loop {
        let e: Event = rx.recv().unwrap();
        info!("event recieved {:?}", e);
    }
}
```
