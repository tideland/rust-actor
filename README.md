# Tideland Rust Actor

[![GitHub release](https://img.shields.io/github/release/tideland/rust-actor.svg)](https://github.com/tideland/rust-actor)
[![GitHub license](https://img.shields.io/badge/license-New%20BSD-blue.svg)](https://raw.githubusercontent.com/tideland/rust-actor/master/LICENSE)

### Description

**Tideland Rust Actor** provides running backend Tokio threads for the sequential execution of closures following the actor model. It allows to do asynchronous and synchronous calls easily into an own context. The type `actor.AsyncActor` is simply processing the closure ony by one and this way removes the need for mutexes and channels. The `actor.MultiActor` is doing the same but allows to additionaly wait for the processing of a sent closure.

I hope you like it. ;)

### Example

```rust
use actor::AsyncActor;

fn main() {
    let actor = actor::AsyncActor::new();

    // The result only contains the success of sending.
    // In case of an error returned by a clossure the internal
    // Actor states is broken and the sending will fail.
    //
    // External reading, setting, and errors have to be handled
    // by the closure itself.
    let result = actor.send(||
        println!("Hello, World!");
    ).await;
}
```

### Contributors

- Frank Mueller (https://github.com/themue / https://github.com/tideland / https://tideland.dev)
